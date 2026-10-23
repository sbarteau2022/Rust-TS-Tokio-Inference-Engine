//! AMST v1: bounded size-prefixed FlatBuffers, with semantic stream verification.
use crate::stream_generated::stream_wire as fb;
use anyhow::{bail, ensure, Result};
use std::io::Write;
pub const MAX_FRAME: usize = 72 * 1024;
pub const MAX_TEXT: usize = 64 * 1024;
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Frame {
    pub version: u16,
    pub kind: u8,
    pub sequence: u64,
    pub index: u32,
    pub token_id: u32,
    pub elapsed_us: u64,
    pub text: String,
    pub model_identity: String,
    pub profile: String,
    pub output_tokens: u32,
    pub stop: u8,
    pub error_code: u32,
}
impl Default for Frame {
    fn default() -> Self {
        Self {
            version: 1,
            kind: 0,
            sequence: 0,
            index: 0,
            token_id: 0,
            elapsed_us: 0,
            text: String::new(),
            model_identity: String::new(),
            profile: String::new(),
            output_tokens: 0,
            stop: 0,
            error_code: 0,
        }
    }
}
impl Frame {
    pub fn metadata(identity: String, profile: String) -> Self {
        Self {
            model_identity: identity,
            profile,
            ..Self::default()
        }
    }
    fn validate(&self) -> Result<()> {
        ensure!(
            self.version == 1 && self.text.len() <= MAX_TEXT && self.profile.len() <= 4096,
            "stream version/string bounds"
        );
        match self.kind {
            0 => ensure!(
                self.model_identity.len() == 64
                    && self
                        .model_identity
                        .bytes()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
                    && !self.profile.is_empty()
                    && self.sequence == 0
                    && self.index == 0
                    && self.token_id == 0
                    && self.elapsed_us == 0
                    && self.text.is_empty()
                    && self.output_tokens == 0
                    && self.stop == 0
                    && self.error_code == 0,
                "metadata fields"
            ),
            1 => ensure!(
                self.model_identity.is_empty()
                    && self.profile.is_empty()
                    && self.sequence > 0
                    && self.index < 128
                    && self.token_id < 248320
                    && self.output_tokens == 0
                    && self.stop == 0
                    && self.error_code == 0,
                "token fields"
            ),
            2 => ensure!(
                self.model_identity.is_empty()
                    && self.profile.is_empty()
                    && self.sequence > 0
                    && self.index == 0
                    && self.token_id == 0
                    && (1..=128).contains(&self.output_tokens)
                    && (1..=2).contains(&self.stop)
                    && self.error_code == 0,
                "finish fields"
            ),
            3 => ensure!(
                self.model_identity.is_empty()
                    && self.profile.is_empty()
                    && self.index == 0
                    && self.token_id == 0
                    && self.output_tokens <= 128
                    && self.stop == 0
                    && (1..=7).contains(&self.error_code)
                    && !self.text.is_empty()
                    && self.text.len() <= 4096,
                "error fields"
            ),
            _ => bail!("unknown stream kind"),
        }
        Ok(())
    }
    pub fn encode(&self) -> Result<Vec<u8>> {
        self.validate()?;
        let mut b = flatbuffers::FlatBufferBuilder::new();
        let text = b.create_string(&self.text);
        let identity = b.create_string(&self.model_identity);
        let profile = b.create_string(&self.profile);
        let root = fb::Frame::create(
            &mut b,
            &fb::FrameArgs {
                version: self.version,
                kind: fb::Kind(self.kind),
                sequence: self.sequence,
                index: self.index,
                token_id: self.token_id,
                elapsed_us: self.elapsed_us,
                text: Some(text),
                model_identity: Some(identity),
                profile: Some(profile),
                output_tokens: self.output_tokens,
                stop: fb::Stop(self.stop),
                error_code: self.error_code,
            },
        );
        fb::finish_size_prefixed_frame_buffer(&mut b, root);
        let bytes = b.finished_data();
        ensure!(bytes.len() <= MAX_FRAME + 4, "frame byte cap");
        Ok(bytes.to_vec())
    }
    pub fn decode(bytes: &[u8]) -> Result<Self> {
        ensure!(
            (16..=MAX_FRAME + 4).contains(&bytes.len()),
            "frame byte bounds"
        );
        ensure!(
            u32::from_le_bytes(bytes[..4].try_into()?) as usize == bytes.len() - 4,
            "frame length mismatch"
        );
        ensure!(
            fb::frame_size_prefixed_buffer_has_identifier(bytes),
            "frame identifier"
        );
        preflight(bytes)?;
        let f = fb::size_prefixed_root_as_frame(bytes)?;

        let out = Self {
            version: f.version(),
            kind: f.kind().0,
            sequence: f.sequence(),
            index: f.index(),
            token_id: f.token_id(),
            elapsed_us: f.elapsed_us(),
            text: f.text().unwrap_or_default().into(),
            model_identity: f.model_identity().unwrap_or_default().into(),
            profile: f.profile().unwrap_or_default().into(),
            output_tokens: f.output_tokens(),
            stop: f.stop().0,
            error_code: f.error_code(),
        };
        out.validate()?;
        Ok(out)
    }
}
// Validate vtable shape before even asking generated table accessors for metadata.
fn preflight(bytes: &[u8]) -> Result<()> {
    fn read<const N: usize>(b: &[u8], p: usize) -> Result<[u8; N]> {
        let end = p
            .checked_add(N)
            .ok_or_else(|| anyhow::anyhow!("wire offset overflow"))?;
        Ok(b.get(p..end)
            .ok_or_else(|| anyhow::anyhow!("wire offset bounds"))?
            .try_into()?)
    }
    let u16at = |p| -> Result<usize> { Ok(u16::from_le_bytes(read(bytes, p)?) as usize) };
    let u32at = |p| -> Result<usize> { Ok(u32::from_le_bytes(read(bytes, p)?) as usize) };
    let root = 4 + u32at(4)?;
    let signed = i32::from_le_bytes(read(bytes, root)?);
    let table = usize::try_from(root as i64 - i64::from(signed))?;
    let vt = u16at(table)?;
    let object = u16at(table + 2)?;
    ensure!(
        root.is_multiple_of(4)
            && table.is_multiple_of(2)
            && (4..=28).contains(&vt)
            && vt.is_multiple_of(2)
            && object >= 4,
        "vtable shape/alignment"
    );
    ensure!(
        table + vt <= bytes.len() && root + object <= bytes.len(),
        "table bounds"
    );
    for (i, width) in [2, 1, 8, 4, 4, 8, 4, 4, 4, 4, 1, 4].into_iter().enumerate() {
        if 4 + i * 2 >= vt {
            break;
        }
        let off = u16at(table + 4 + i * 2)?;
        if off == 0 {
            continue;
        }
        ensure!(
            off >= 4 && off + width <= object && (root + off).is_multiple_of(width),
            "field bounds/alignment"
        );
        if (6..=8).contains(&i) {
            let relative = u32at(root + off)?;
            let start = root + off + relative;
            let len = u32at(start)?;
            ensure!(
                relative >= 4
                    && start.is_multiple_of(4)
                    && len <= MAX_TEXT
                    && start + 4 + len < bytes.len()
                    && bytes[start + 4 + len] == 0,
                "string bounds/terminator"
            );
            std::str::from_utf8(&bytes[start + 4..start + 4 + len])?;
        }
    }
    Ok(())
}
#[derive(Default)]
struct State {
    metadata: bool,
    terminal: bool,
    sequence: u64,
    count: u32,
    last_us: u64,
    text_bytes: usize,
    frames: usize,
}
impl State {
    fn accept(&mut self, f: &Frame) -> Result<()> {
        f.validate()?;
        ensure!(!self.terminal && self.frames < 130, "terminal/frame cap");
        ensure!(
            self.text_bytes + f.text.len() <= 1024 * 1024,
            "stream text cap"
        );
        if f.kind == 0 {
            ensure!(!self.metadata && self.frames == 0, "metadata order");
            self.metadata = true;
        } else {
            ensure!(
                self.metadata && f.elapsed_us >= self.last_us,
                "missing metadata/time regression"
            );
            match f.kind {
                1 => {
                    ensure!(
                        f.index == self.count
                            && self.count < 128
                            && (self.sequence == 0 || f.sequence == self.sequence),
                        "token sequence/index"
                    );
                    self.sequence = f.sequence;
                    self.count += 1;
                }
                2 => {
                    ensure!(
                        f.sequence == self.sequence
                            && f.output_tokens == self.count
                            && self.count > 0,
                        "finish count/sequence"
                    );
                    self.terminal = true;
                }
                3 => {
                    ensure!(
                        f.output_tokens == self.count && f.sequence == self.sequence,
                        "error count/sequence"
                    );
                    self.terminal = true;
                }
                _ => unreachable!(),
            }
            self.last_us = f.elapsed_us;
        }
        self.text_bytes += f.text.len();
        self.frames += 1;
        Ok(())
    }
}
#[derive(Default)]
pub struct Decoder {
    pending: Vec<u8>,
    state: State,
    poisoned: bool,
}
impl Decoder {
    pub fn push(&mut self, mut input: &[u8]) -> Result<Vec<Frame>> {
        ensure!(!self.poisoned, "stream parser poisoned");
        self.poisoned = true;
        let mut out = Vec::new();
        while !input.is_empty() {
            ensure!(!self.state.terminal, "bytes after terminal");
            let target = if self.pending.len() < 4 {
                4
            } else {
                let n = u32::from_le_bytes(self.pending[..4].try_into()?) as usize;
                ensure!((12..=MAX_FRAME).contains(&n), "frame length cap");
                n + 4
            };
            let n = (target - self.pending.len()).min(input.len());
            self.pending.extend_from_slice(&input[..n]);
            input = &input[n..];
            if self.pending.len() == 4 {
                let length = u32::from_le_bytes(self.pending[..4].try_into()?) as usize;
                ensure!((12..=MAX_FRAME).contains(&length), "frame length cap");
            }
            if self.pending.len() == target && target > 4 {
                let f = Frame::decode(&self.pending)?;
                self.state.accept(&f)?;
                out.push(f);
                self.pending.clear();
            }
        }
        self.poisoned = false;
        Ok(out)
    }
    pub fn finish(&mut self) -> Result<()> {
        ensure!(!self.poisoned, "stream parser poisoned");
        self.poisoned = true;
        ensure!(
            self.pending.is_empty() && self.state.terminal,
            "unfinished/truncated stream"
        );
        Ok(())
    }
}
pub struct Encoder<W: Write> {
    writer: W,
    state: State,
    poisoned: bool,
}
impl<W: Write> Encoder<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            state: State::default(),
            poisoned: false,
        }
    }
    pub fn write(&mut self, f: &Frame) -> Result<()> {
        ensure!(!self.poisoned, "stream writer poisoned");
        self.poisoned = true;
        self.state.accept(f)?;
        let bytes = f.encode()?;
        self.writer.write_all(&bytes)?;
        self.writer.flush()?;
        self.poisoned = false;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn frames() -> Vec<Frame> {
        vec![
            Frame::metadata("a".repeat(64), "experimental-test".into()),
            Frame {
                kind: 1,
                sequence: 9,
                token_id: 42,
                text: "é😀\n".into(),
                elapsed_us: 5,
                ..Frame::default()
            },
            Frame {
                kind: 2,
                sequence: 9,
                output_tokens: 1,
                stop: 1,
                elapsed_us: 8,
                text: "�".into(),
                ..Frame::default()
            },
        ]
    }
    #[test]
    fn fragmented_coalesced_and_terminals() -> Result<()> {
        let expected = frames();
        let bytes = expected
            .iter()
            .map(Frame::encode)
            .collect::<Result<Vec<_>>>()?
            .concat();
        for chunk in 1..=bytes.len() {
            let mut d = Decoder::default();
            let mut got = Vec::new();
            for c in bytes.chunks(chunk) {
                got.extend(d.push(c)?);
            }
            d.finish()?;
            assert_eq!(got, expected);
            assert!(d.push(&[]).is_err());
        }
        let mut e = Encoder::new(Vec::new());
        for f in &expected {
            e.write(f)?;
        }
        assert!(e.write(&expected[2]).is_err());
        assert!(e.write(&expected[0]).is_err());
        Ok(())
    }
    #[test]
    fn invalid_sequences_versions_eof_and_bounds() -> Result<()> {
        let fs = frames();
        let mut bad = fs[1].clone();
        bad.index = 1;
        for series in [
            vec![fs[1].clone()],
            vec![fs[0].clone(), fs[0].clone()],
            vec![fs[0].clone(), bad],
            vec![fs[0].clone(), fs[2].clone()],
        ] {
            let mut d = Decoder::default();
            assert!(d
                .push(
                    &series
                        .iter()
                        .map(Frame::encode)
                        .collect::<Result<Vec<_>>>()?
                        .concat()
                )
                .is_err());
            assert!(d.push(&[]).is_err());
        }
        let bytes = fs
            .iter()
            .map(Frame::encode)
            .collect::<Result<Vec<_>>>()?
            .concat();
        for n in 0..bytes.len() {
            let mut d = Decoder::default();
            if d.push(&bytes[..n]).is_ok() {
                assert!(d.finish().is_err());
            }
        }
        for n in [0u32, 11, MAX_FRAME as u32 + 1, u32::MAX] {
            assert!(Decoder::default().push(&n.to_le_bytes()).is_err());
        }
        let mut f = fs[1].clone();
        f.version = 2;
        assert!(f.encode().is_err());
        f.version = 1;
        f.kind = 255;
        assert!(f.encode().is_err());
        Ok(())
    }
    #[test]
    fn errors_limits_and_broken_writer_are_explicit() -> Result<()> {
        let fs = frames();
        for partial in [false, true] {
            let mut series = vec![fs[0].clone()];
            if partial {
                series.push(fs[1].clone());
            }
            series.push(Frame {
                kind: 3,
                sequence: if partial { 9 } else { 0 },
                output_tokens: u32::from(partial),
                elapsed_us: 8,
                error_code: 2,
                text: "deadline".into(),
                ..Frame::default()
            });
            let mut d = Decoder::default();
            d.push(
                &series
                    .iter()
                    .map(Frame::encode)
                    .collect::<Result<Vec<_>>>()?
                    .concat(),
            )?;
            d.finish()?;
        }
        let mut d = Decoder::default();
        d.push(&fs[0].encode()?)?;
        for i in 0..128 {
            d.push(
                &Frame {
                    kind: 1,
                    sequence: 9,
                    index: i,
                    elapsed_us: u64::from(i),
                    ..Frame::default()
                }
                .encode()?,
            )?;
        }
        assert!(d.push(&fs[1].encode()?).is_err());
        let mut d = Decoder::default();
        d.push(&fs[0].encode()?)?;
        for i in 0..16 {
            d.push(
                &Frame {
                    kind: 1,
                    sequence: 9,
                    index: i,
                    text: "x".repeat(MAX_TEXT),
                    ..Frame::default()
                }
                .encode()?,
            )?;
        }
        assert!(d
            .push(
                &Frame {
                    kind: 1,
                    sequence: 9,
                    index: 16,
                    text: "x".into(),
                    ..Frame::default()
                }
                .encode()?
            )
            .is_err());
        struct Broken {
            left: usize,
        }
        impl std::io::Write for Broken {
            fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
                if self.left == 0 {
                    return Err(std::io::ErrorKind::BrokenPipe.into());
                }
                let n = self.left.min(b.len());
                self.left -= n;
                Ok(n)
            }
            fn flush(&mut self) -> std::io::Result<()> {
                Ok(())
            }
        }
        let mut writer = Encoder::new(Broken { left: 8 });
        assert!(writer.write(&fs[0]).is_err());
        assert!(writer.write(&fs[0]).is_err());
        Ok(())
    }
    #[test]
    fn malformed_mutations_never_panic_and_poison_is_sticky() -> Result<()> {
        let bytes = frames()
            .iter()
            .map(Frame::encode)
            .collect::<Result<Vec<_>>>()?
            .concat();
        for i in 0..bytes.len() {
            for value in [0, 1, 127, 255] {
                let mut v = bytes.clone();
                v[i] = value;
                let mut d = Decoder::default();
                if d.push(&v).is_err() {
                    assert!(d.push(&[]).is_err());
                } else {
                    let _ = d.finish();
                }
            }
        }
        Ok(())
    }
}
