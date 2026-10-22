//! Integer host reference codecs; packed native attention is a separate backend capability.
use super::{ByteBudget, Charge};
use anyhow::{ensure, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::sync::Arc;
const Q31: i128 = 1 << 31;
const Q30: i128 = 1 << 30;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TurboProfile {
    pub version: u32,
    pub width: usize,
    pub bits: u8,
    /// Row-major orthogonal rotation in signed Q2.30. No implicit random seed.
    pub rotation_q30: Vec<i32>,
    /// Normalized scalar reconstruction levels in increasing signed Q1.31 order.
    pub centers_q31: Vec<i32>,
}
impl TurboProfile {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.version == 1,
            "unsupported TurboQuant-MSE profile version"
        );
        ensure!(
            (2..=256).contains(&self.width) && (2..=4).contains(&self.bits),
            "codec dimensions/bits"
        );
        ensure!(
            self.rotation_q30.len() == self.width * self.width
                && self.centers_q31.len() == 1 << self.bits,
            "profile shape"
        );
        ensure!(
            self.centers_q31.windows(2).all(|v| v[0] < v[1])
                && self.centers_q31[0] < 0
                && *self.centers_q31.last().unwrap() > 0,
            "ordered signed codebook required"
        );
        ensure!(
            self.rotation_q30
                .iter()
                .all(|v| i64::from(*v).abs() <= 1 << 30),
            "rotation coefficient range"
        );
        // Bound the quantized orthogonality defect; neither this nor codebook order proves model quality.
        for a in 0..self.width {
            for b in 0..=a {
                let dot: i128 = (0..self.width)
                    .map(|j| {
                        i128::from(self.rotation_q30[a * self.width + j])
                            * i128::from(self.rotation_q30[b * self.width + j])
                    })
                    .sum();
                let expected = if a == b { Q30 * Q30 } else { 0 };
                ensure!(
                    (dot - expected).abs() <= (self.width as i128) * Q30 * 4,
                    "rotation is not orthogonal within fixed-point tolerance"
                );
            }
        }
        Ok(())
    }
}
#[derive(Clone, Debug)]
pub struct Codec {
    width: usize,
    profile: Option<Arc<TurboProfile>>,
    fingerprint: [u8; 32],
}
#[derive(Debug)]
pub struct Frame {
    fingerprint: [u8; 32],
    bytes: Vec<u8>,
    _charge: Charge,
}
impl Frame {
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn fingerprint(&self) -> [u8; 32] {
        self.fingerprint
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct Decoded {
    pub keys: Vec<i32>,
    pub values: Vec<i32>,
    pub saturated_values: usize,
}
impl Codec {
    pub fn regular(width: usize) -> Result<Self> {
        ensure!((2..=256).contains(&width), "KV width outside 2..256");
        let mut hash = Sha256::new();
        hash.update(b"regular-q31-le-v1");
        hash.update((width as u32).to_le_bytes());
        Ok(Self {
            width,
            profile: None,
            fingerprint: hash.finalize().into(),
        })
    }
    pub fn turbo_mse(profile: TurboProfile) -> Result<Self> {
        profile.validate()?;
        let mut hash = Sha256::new();
        hash.update(b"regular-k-turboquant-mse-v-q31-v1");
        hash.update(serde_json::to_vec(&profile)?);
        Ok(Self {
            width: profile.width,
            profile: Some(Arc::new(profile)),
            fingerprint: hash.finalize().into(),
        })
    }
    pub fn width(&self) -> usize {
        self.width
    }
    pub fn fingerprint(&self) -> [u8; 32] {
        self.fingerprint
    }
    pub fn experimental(&self) -> bool {
        self.profile.is_some()
    }
    pub fn frame_bytes(&self) -> usize {
        self.width * 4
            + self.profile.as_ref().map_or(self.width * 4, |p| {
                8 + (self.width * usize::from(p.bits)).div_ceil(8)
            })
    }
    pub fn reserve_frame(&self, budget: &ByteBudget) -> Result<Charge> {
        budget.reserve(self.frame_bytes())
    }
    pub fn encode(&self, keys: &[i32], values: &[i32], budget: &ByteBudget) -> Result<Arc<Frame>> {
        self.encode_reserved(keys, values, self.reserve_frame(budget)?)
    }
    pub(crate) fn encode_reserved(
        &self,
        keys: &[i32],
        values: &[i32],
        charge: Charge,
    ) -> Result<Arc<Frame>> {
        ensure!(
            keys.len() == self.width && values.len() == self.width,
            "KV frame shape"
        );
        let mut bytes = Vec::with_capacity(self.frame_bytes());
        for v in keys {
            bytes.extend(v.to_le_bytes());
        }
        if let Some(p) = &self.profile {
            let norm = isqrt(values.iter().map(|v| i128::from(*v).pow(2) as u128).sum());
            bytes.extend((norm as u64).to_le_bytes());
            let start = bytes.len();
            bytes.resize(self.frame_bytes(), 0);
            for row in 0..self.width {
                let dot: i128 = values
                    .iter()
                    .enumerate()
                    .map(|(j, v)| i128::from(p.rotation_q30[row * self.width + j]) * i128::from(*v))
                    .sum();
                let coordinate = if norm == 0 {
                    0
                } else {
                    round_even(dot * Q31, (norm as i128) * Q30)
                };
                let index = p
                    .centers_q31
                    .iter()
                    .enumerate()
                    .min_by_key(|(_, c)| (coordinate - i128::from(**c)).abs())
                    .unwrap()
                    .0;
                for bit in 0..usize::from(p.bits) {
                    let position = row * usize::from(p.bits) + bit;
                    bytes[start + position / 8] |= (((index >> bit) & 1) as u8) << (position % 8);
                }
            }
        } else {
            for v in values {
                bytes.extend(v.to_le_bytes());
            }
        }
        Ok(Arc::new(Frame {
            fingerprint: self.fingerprint,
            bytes,
            _charge: charge,
        }))
    }
    pub fn decode(&self, frame: &Frame) -> Result<Decoded> {
        ensure!(
            frame.fingerprint == self.fingerprint,
            "KV codec identity mismatch"
        );
        self.decode_bytes(frame.bytes())
    }
    /// Untrusted imported payloads require identity checking by the caller before this primitive.
    pub fn decode_bytes(&self, bytes: &[u8]) -> Result<Decoded> {
        ensure!(bytes.len() == self.frame_bytes(), "KV payload length");
        let keys = bytes[..self.width * 4]
            .as_chunks::<4>()
            .0
            .iter()
            .map(|b| i32::from_le_bytes(*b))
            .collect();
        let mut saturated_values = 0;
        let values = if let Some(p) = &self.profile {
            let start = self.width * 4;
            let norm = u64::from_le_bytes(bytes[start..start + 8].try_into().unwrap());
            ensure!(
                u128::from(norm) <= isqrt((self.width as u128) * (1u128 << 62)),
                "KV norm out of range"
            );
            let packed = &bytes[start + 8..];
            let used = self.width * usize::from(p.bits);
            if !used.is_multiple_of(8) {
                ensure!(
                    packed.last().unwrap() >> (used % 8) == 0,
                    "noncanonical packed tail"
                );
            }
            let mut coordinates = Vec::with_capacity(self.width);
            for row in 0..self.width {
                let mut index = 0;
                for bit in 0..usize::from(p.bits) {
                    let position = row * usize::from(p.bits) + bit;
                    index |= usize::from((packed[position / 8] >> (position % 8)) & 1) << bit;
                }
                coordinates.push(p.centers_q31[index]);
            }
            (0..self.width)
                .map(|j| {
                    let dot: i128 = coordinates
                        .iter()
                        .enumerate()
                        .map(|(row, v)| {
                            i128::from(p.rotation_q30[row * self.width + j]) * i128::from(*v)
                        })
                        .sum();
                    let raw = round_even(dot * i128::from(norm), Q30 * Q31);
                    if raw < i128::from(i32::MIN) || raw > i128::from(i32::MAX) {
                        saturated_values += 1;
                    }
                    raw.clamp(i128::from(i32::MIN), i128::from(i32::MAX)) as i32
                })
                .collect()
        } else {
            bytes[self.width * 4..]
                .as_chunks::<4>()
                .0
                .iter()
                .map(|b| i32::from_le_bytes(*b))
                .collect()
        };
        Ok(Decoded {
            keys,
            values,
            saturated_values,
        })
    }
}
fn round_even(n: i128, d: i128) -> i128 {
    debug_assert!(d > 0);
    let q = n.div_euclid(d);
    let r = n.rem_euclid(d);
    q + i128::from(r * 2 > d || (r * 2 == d && q % 2 != 0))
}
fn isqrt(n: u128) -> u128 {
    if n == 0 {
        return 0;
    }
    let mut x = 1u128 << (128 - n.leading_zeros()).div_ceil(2);
    loop {
        let next = (x + n / x) / 2;
        if next >= x {
            return x;
        }
        x = next;
    }
}
