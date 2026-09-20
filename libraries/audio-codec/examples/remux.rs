//! Losslessly remux an Ogg Opus clip for playback with Apple's native players.
fn main() -> anyhow::Result<()> {
    let args: Vec<_> = std::env::args_os().collect();
    anyhow::ensure!(args.len() == 3, "usage: remux INPUT.ogg OUTPUT.caf");
    let bytes = std::fs::read(&args[1])?;
    std::fs::write(&args[2], audio_codec::ogg_opus_to_caf(&bytes)?)?;
    Ok(())
}
