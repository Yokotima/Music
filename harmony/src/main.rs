mod files;
mod ui;
mod utils;
mod audio;
mod sequencer;

use anyhow::Result;
use std::io::{self, BufRead};

fn main() -> Result<()>
{
    let args: Vec<String> = std::env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("default");

    match command
    {
        "test_notes" =>
        {
            println!("=== HarmonyStudio — Test all piano notes ===");
            audio::piano_test::run_all_notes()?;
        }

        "test_note" =>
        {
            let note_name = args.get(2).map(|s| s.as_str()).unwrap_or("A4");
            println!("=== HarmonyStudio — Test note: {} ===", note_name);
            audio::piano_test::run_single_note(note_name)?;
        }
        "test_music" =>
        {
            println!("=== HarmonyStudio — Test Music===");
            let _engine = audio::audio_engine::AudioEngine::start()?;
            println!("Press ENTER to stop.");
            io::stdin().lock().lines().next();
            println!("Shutting down.");
        }

        _ =>
        {
            println!("=== HarmonyStudio ===");
            println!("Commands:");
            println!("  cargo run test_notes   -> test all 88 piano keys");
            println!("  cargo run test_note A4 -> test one specific note");
            println!("  cargo run test_music   -> test a music");
            println!();
        }
    }

    Ok(())
}
