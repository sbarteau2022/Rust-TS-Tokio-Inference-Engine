use anyhow::{Context, Result};
use rust_server::serving::stream_wire::{Encoder, Frame};
fn main() -> Result<()> {
    let path = std::env::args()
        .nth(1)
        .context("stream_wire_fixture NEW_FILE")?;
    let file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    let mut writer = Encoder::new(file);
    writer.write(&Frame::metadata(
        "a".repeat(64),
        "cross-language-experimental".into(),
    ))?;
    writer.write(&Frame {
        kind: 1,
        sequence: 9,
        token_id: 42,
        text: "\u{feff}é😀\n".into(),
        elapsed_us: 5,
        ..Frame::default()
    })?;
    writer.write(&Frame {
        kind: 2,
        sequence: 9,
        output_tokens: 1,
        stop: 1,
        elapsed_us: 8,
        text: "�".into(),
        ..Frame::default()
    })?;
    Ok(())
}
