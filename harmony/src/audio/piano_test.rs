use super::note::{MidiNote, ALL_NOTES};
use super::play::play;
use super::instruments::InstrumentKind;

pub fn run_all_notes()
{
    println!("Playing all 88 notes (0.4s each)...\n");

    for &note in &ALL_NOTES
    {
        play(note, InstrumentKind::Piano, 0.4);
    }

    println!("\nAll notes played.");
}
