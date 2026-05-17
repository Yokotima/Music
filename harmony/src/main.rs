mod files;
mod ui;
mod audio;
mod sequencer;

use anyhow::Result;

fn main() -> Result<()> {
    if let Err(e) = ui::window::window() {
        eprintln!("Window error: {e}");
    }
    Ok(())
}
