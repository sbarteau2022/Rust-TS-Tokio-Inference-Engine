//! Typed model control IR. No text parsing, lexical conversion, or model-vocabulary substitution.
use anyhow::{ensure, Result};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepKind {
    Prefill,
    Decode,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TokenPosition(usize);
impl TokenPosition {
    pub fn get(self) -> usize {
        self.0
    }
}
#[derive(Clone, Debug)]
pub struct ModelStepIr {
    kind: StepKind,
    position: TokenPosition,
    vocabulary: u32,
    tokens: Box<[u32]>,
}
impl ModelStepIr {
    pub fn compile(
        kind: StepKind,
        position: usize,
        tokens: Vec<u32>,
        vocabulary: u32,
    ) -> Result<Self> {
        ensure!(
            vocabulary > 0
                && vocabulary <= 300000
                && !tokens.is_empty()
                && tokens.len() <= 8
                && tokens.iter().all(|t| *t < vocabulary),
            "IR token geometry"
        );
        ensure!(
            kind != StepKind::Decode || tokens.len() == 1,
            "decode IR requires one token"
        );
        ensure!(
            position.checked_add(tokens.len()).is_some_and(|n| n <= 512),
            "IR context bound"
        );
        Ok(Self {
            kind,
            position: TokenPosition(position),
            vocabulary,
            tokens: tokens.into_boxed_slice(),
        })
    }
    pub fn kind(&self) -> StepKind {
        self.kind
    }
    pub fn position(&self) -> TokenPosition {
        self.position
    }
    pub fn len(&self) -> usize {
        self.tokens.len()
    }
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }
    pub fn vocabulary(&self) -> u32 {
        self.vocabulary
    }
    pub(super) fn ffi_ids(&self) -> &[u32] {
        &self.tokens
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mixer {
    GatedDelta,
    FullAttention,
}
/// C ABI lowers typed mixer variants to checked integer tags; never transmute an enum from C.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct LayerIr {
    index: u32,
    mixer: u32,
}
impl LayerIr {
    pub fn index(&self) -> usize {
        self.index as usize
    }
    pub fn mixer(&self) -> Mixer {
        if self.mixer == 0 {
            Mixer::GatedDelta
        } else {
            Mixer::FullAttention
        }
    }
}
#[derive(Clone, Debug)]
pub struct HybridProgram {
    layers: Box<[LayerIr]>,
}
impl HybridProgram {
    pub fn compile(layers: usize, interval: usize) -> Result<Self> {
        ensure!(
            (1..=64).contains(&layers)
                && interval > 0
                && interval <= layers
                && layers.is_multiple_of(interval),
            "IR hybrid program geometry"
        );
        Ok(Self {
            layers: (0..layers)
                .map(|i| LayerIr {
                    index: i as u32,
                    mixer: u32::from((i + 1) % interval == 0),
                })
                .collect(),
        })
    }
    pub fn layers(&self) -> &[LayerIr] {
        &self.layers
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn typed_program_and_steps_validate_bounds() -> Result<()> {
        let p = HybridProgram::compile(40, 4)?;
        assert_eq!(p.layers().len(), 40);
        assert_eq!(
            p.layers()
                .iter()
                .filter(|l| l.mixer() == Mixer::FullAttention)
                .count(),
            10
        );
        for (i, l) in p.layers().iter().enumerate() {
            assert_eq!(l.index(), i);
        }
        assert_eq!(std::mem::size_of::<LayerIr>(), 8);
        assert!(HybridProgram::compile(40, 0).is_err());
        assert!(HybridProgram::compile(41, 4).is_err());
        let step = ModelStepIr::compile(StepKind::Decode, 511, vec![99], 100)?;
        assert_eq!(step.position().get(), 511);
        assert_eq!(step.ffi_ids(), &[99]);
        for (kind, pos, tokens, vocab) in [
            (StepKind::Decode, 0, vec![1, 2], 100),
            (StepKind::Prefill, 512, vec![1], 100),
            (StepKind::Prefill, usize::MAX, vec![1], 100),
            (StepKind::Prefill, 0, vec![], 100),
            (StepKind::Prefill, 0, vec![100], 100),
            (StepKind::Prefill, 0, vec![1; 9], 100),
            (StepKind::Decode, 0, vec![1], 0),
        ] {
            assert!(ModelStepIr::compile(kind, pos, tokens, vocab).is_err());
        }
        Ok(())
    }
    #[test]
    fn core_runtime_has_no_lexical_dependency() {
        for text in [include_str!("native_qwen.rs"), include_str!("scheduler.rs")] {
            for forbidden in [
                "NativeTokenizer",
                "render_chat",
                "native_tokenizer",
                "tokenizer::",
            ] {
                assert!(
                    !text.contains(forbidden),
                    "lexical dependency in runtime: {forbidden}"
                );
            }
        }
    }
}
