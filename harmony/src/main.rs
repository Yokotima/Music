// src/main.rs
mod audio;
mod files;
mod sequencer;
mod ui;
mod utils;

use anyhow::Result;
use std::io::{self, BufRead};

fn main() -> Result<()> {
    println!("=== HarmonyStudio ===");

    let _engine = audio::audio_engine::AudioEngine::start()?;

    println!("Press ENTER to stop.");
    io::stdin().lock().lines().next();

    println!("Shutting down.");
    Ok(())
}
