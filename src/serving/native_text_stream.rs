//! Append-only tokenizer output; incomplete UTF-8 remains buffered until valid or final.
use anyhow::{ensure, Result};
type Decoder<'a> = tokenizers::DecodeStream<
    'a,
    tokenizers::models::ModelWrapper,
    tokenizers::normalizers::NormalizerWrapper,
    tokenizers::pre_tokenizers::PreTokenizerWrapper,
    tokenizers::processors::PostProcessorWrapper,
    tokenizers::decoders::DecoderWrapper,
>;
pub struct NativeTextStream<'a> {
    tokenizer: &'a tokenizers::Tokenizer,
    manifest: Option<&'a super::weights::Manifest>,
    decoder: Decoder<'a>,
    ids: Vec<u32>,
    published: String,
    closed: bool,
    poisoned: bool,
}
impl<'a> NativeTextStream<'a> {
    pub(super) fn new(tokenizer: &'a tokenizers::Tokenizer) -> Self {
        Self {
            tokenizer,
            manifest: None,
            decoder: tokenizer.decode_stream(true),
            ids: Vec::new(),
            published: String::new(),
            closed: false,
            poisoned: false,
        }
    }
    pub(super) fn with_manifest(mut self, manifest: &'a super::weights::Manifest) -> Self {
        self.manifest = Some(manifest);
        self
    }
    pub fn push(&mut self, id: u32) -> Result<Option<String>> {
        ensure!(
            !self.closed && !self.poisoned,
            "text stream closed/poisoned"
        );
        self.poisoned = true;
        ensure!(
            self.ids.len() < 512 && self.tokenizer.id_to_token(id).is_some(),
            "text stream token cap/unknown ID"
        );
        let delta = self
            .decoder
            .step(id)
            .map_err(|e| anyhow::anyhow!("stream decode: {e}"))?;
        self.ids.push(id);
        if let Some(s) = &delta {
            self.published.push_str(s);
        }
        self.poisoned = false;
        Ok(delta)
    }
    pub fn finish(&mut self) -> Result<String> {
        ensure!(
            !self.closed && !self.poisoned,
            "text stream closed/poisoned"
        );
        self.closed = true;
        if let Some(manifest) = self.manifest {
            manifest.verify_stamps()?;
        }
        let full = self
            .tokenizer
            .decode(&self.ids, true)
            .map_err(|e| anyhow::anyhow!("final stream decode: {e}"))?;
        ensure!(
            full.starts_with(&self.published),
            "text decoder revised published prefix"
        );
        let tail = full[self.published.len()..].to_owned();
        self.published = full;
        Ok(tail)
    }
    pub fn text(&self) -> &str {
        &self.published
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn tokenizer() -> tokenizers::Tokenizer {
        let vocab = [
            ("x", 0),
            ("<0xC3>", 1),
            ("<0xA9>", 2),
            ("<0xF0>", 3),
            ("<0x9F>", 4),
            ("<0x98>", 5),
            ("<0x80>", 6),
            ("�", 7),
            ("\n ", 8),
        ];
        let model = tokenizers::models::bpe::BPE::builder()
            .vocab_and_merges(
                vocab
                    .into_iter()
                    .map(|(s, id)| (s.to_string(), id))
                    .collect::<tokenizers::models::bpe::Vocab>(),
                vec![],
            )
            .byte_fallback(true)
            .build()
            .unwrap();
        let mut t = tokenizers::Tokenizer::new(model);
        t.with_decoder(Some(
            tokenizers::decoders::byte_fallback::ByteFallback::default(),
        ));
        t.add_special_tokens([tokenizers::AddedToken::from("<end>", true)])
            .unwrap();
        t
    }
    #[test]
    fn unicode_and_terminal_replacement_match_canonical() -> Result<()> {
        let t = tokenizer();
        let special = t.token_to_id("<end>").unwrap();
        for ids in [
            vec![1, 2],
            vec![3, 4, 5, 6],
            vec![7, 0, 8, special, 1, 2],
            vec![1],
            vec![3, 4],
            vec![special],
            vec![],
        ] {
            let expected = t.decode(&ids, true).unwrap();
            let mut d = NativeTextStream::new(&t);
            let mut out = String::new();
            for id in &ids {
                if let Some(s) = d.push(*id)? {
                    out.push_str(&s);
                    assert!(expected.starts_with(&out));
                }
            }
            out.push_str(&d.finish()?);
            assert_eq!(out, expected);
            assert_eq!(d.text(), expected);
            assert!(d.finish().is_err());
            assert!(d.push(0).is_err());
        }
        let mut d = NativeTextStream::new(&t);
        assert!(d.push(1)?.is_none());
        assert_eq!(d.push(2)?.as_deref(), Some("é"));
        Ok(())
    }
    #[test]
    fn rewriting_decoder_fails_and_poison_is_sticky() -> Result<()> {
        let mut t = tokenizer();
        t.with_decoder(Some(tokenizers::decoders::sequence::Sequence::new(vec![
            tokenizers::decoders::fuse::Fuse::new().into(),
            tokenizers::normalizers::replace::Replace::new("xx", "y")
                .unwrap()
                .into(),
        ])));
        let mut d = NativeTextStream::new(&t);
        assert_eq!(d.push(0)?.as_deref(), Some("x"));
        assert!(d.push(0)?.is_none());
        assert!(d.push(0).is_err());
        assert!(d.push(0).is_err());
        assert!(d.finish().is_err());
        let mut d = NativeTextStream::new(&t);
        d.push(0)?;
        d.push(0)?;
        assert!(d.finish().is_err());
        assert!(d.push(0).is_err());
        Ok(())
    }
    #[test]
    fn unknown_and_limit_poison_stream() -> Result<()> {
        let t = tokenizer();
        let mut d = NativeTextStream::new(&t);
        assert!(d.push(u32::MAX).is_err());
        assert!(d.push(0).is_err());
        assert!(d.finish().is_err());
        let mut d = NativeTextStream::new(&t);
        for _ in 0..512 {
            d.push(0)?;
        }
        assert!(d.push(0).is_err());
        assert!(d.finish().is_err());
        Ok(())
    }
}
