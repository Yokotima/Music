#![allow(dead_code)]
use super::note::ALL_NOTES;
use super::play::play;
use super::instruments::InstrumentKind;
use super::effects::EffectMode;

pub fn run_all_notes()
{
    println!("Playing all 88 notes (0.4s each)...\n");

    for &note in &ALL_NOTES
    {
        play(note, InstrumentKind::Piano, 0.4, EffectMode::None);
    }

    println!("\nAll notes played.");
}
