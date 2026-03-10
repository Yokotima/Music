/// audio/piano_test.rs
///
/// Test functions for piano notes.
/// Called from main.rs via command line arguments.
///
///   cargo run test_notes      → plays all 88 keys A0 to C8
///   cargo run test_note A4    → plays one specific note

use anyhow::{anyhow, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, SampleRate, StreamConfig};
use std::sync::{Arc, Mutex};

use crate::sequencer::sequencer::StepSequencer;
use super::instruments::InstrumentKind;
use super::piano::{play_sound, stop_sound, PianoNote};

// All 88 notes in order A0 → C8
const ALL_NOTES: [PianoNote; 88] = [
    PianoNote::A0,  PianoNote::As0, PianoNote::B0,
    PianoNote::C1,  PianoNote::Cs1, PianoNote::D1,  PianoNote::Ds1,
    PianoNote::E1,  PianoNote::F1,  PianoNote::Fs1, PianoNote::G1,
    PianoNote::Gs1, PianoNote::A1,  PianoNote::As1, PianoNote::B1,
    PianoNote::C2,  PianoNote::Cs2, PianoNote::D2,  PianoNote::Ds2,
    PianoNote::E2,  PianoNote::F2,  PianoNote::Fs2, PianoNote::G2,
    PianoNote::Gs2, PianoNote::A2,  PianoNote::As2, PianoNote::B2,
    PianoNote::C3,  PianoNote::Cs3, PianoNote::D3,  PianoNote::Ds3,
    PianoNote::E3,  PianoNote::F3,  PianoNote::Fs3, PianoNote::G3,
    PianoNote::Gs3, PianoNote::A3,  PianoNote::As3, PianoNote::B3,
    PianoNote::C4,  PianoNote::Cs4, PianoNote::D4,  PianoNote::Ds4,
    PianoNote::E4,  PianoNote::F4,  PianoNote::Fs4, PianoNote::G4,
    PianoNote::Gs4, PianoNote::A4,  PianoNote::As4, PianoNote::B4,
    PianoNote::C5,  PianoNote::Cs5, PianoNote::D5,  PianoNote::Ds5,
    PianoNote::E5,  PianoNote::F5,  PianoNote::Fs5, PianoNote::G5,
    PianoNote::Gs5, PianoNote::A5,  PianoNote::As5, PianoNote::B5,
    PianoNote::C6,  PianoNote::Cs6, PianoNote::D6,  PianoNote::Ds6,
    PianoNote::E6,  PianoNote::F6,  PianoNote::Fs6, PianoNote::G6,
    PianoNote::Gs6, PianoNote::A6,  PianoNote::As6, PianoNote::B6,
    PianoNote::C7,  PianoNote::Cs7, PianoNote::D7,  PianoNote::Ds7,
    PianoNote::E7,  PianoNote::F7,  PianoNote::Fs7, PianoNote::G7,
    PianoNote::Gs7, PianoNote::A7,  PianoNote::As7, PianoNote::B7,
    PianoNote::C8,
];

// ─────────────────────────────────────────────────────────────────────────────
// run_all_notes — plays every key from A0 to C8
// ─────────────────────────────────────────────────────────────────────────────

// Plays all 88 piano keys one by one, 0.4s each.
// Creates a sequencer with one Piano track — play_sound triggers notes on it.
pub fn run_all_notes() -> Result<()>
{
    // Sequencer with 1 step — we don't use the grid here, just the engine inside
    let seq = Arc::new(Mutex::new(StepSequencer::new(120.0, 1, 44_100)));

    // Add one Piano track — play_sound will find it automatically
    {
        let mut s = seq.lock().unwrap();
        let idx = s.add_track(InstrumentKind::Piano, 60);
        // Dry — no effects so every note rings cleanly
        s.tracks[idx].engine.fx.reverb.set_wet_mix(0.0);
        s.tracks[idx].engine.fx.delay.set_wet_mix(0.0);
    }

    let stream = build_stream(Arc::clone(&seq))?;
    stream.play()?;
    std::thread::sleep(std::time::Duration::from_millis(100));

    println!("Playing all 88 notes (0.4s each)...\n");

    for &note in &ALL_NOTES
    {
        {
            let mut s = seq.lock().unwrap();
            play_sound(note, 0.75, &mut s);
        }

        // Hold the note for 300ms
        std::thread::sleep(std::time::Duration::from_millis(300));

        {
            let mut s = seq.lock().unwrap();
            stop_sound(note, &mut s);
        }

        // Gap between notes: 100ms
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    std::thread::sleep(std::time::Duration::from_millis(500));
    println!("\nAll notes played.");

    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// run_single_note — plays one note by name
// ─────────────────────────────────────────────────────────────────────────────

// Plays one specific note by name for 2 seconds.
// Accepts names like: A4, C4, Cs4 (C#4), Fs3 (F#3), Bb4, etc.
pub fn run_single_note(name: &str) -> Result<()>
{
    let note = parse_note(name)
        .ok_or_else(|| anyhow!(
            "Unknown note: '{}'\nValid examples: A4, C4, Cs4, Fs3, B2, C8",
            name
        ))?;

    let seq = Arc::new(Mutex::new(StepSequencer::new(120.0, 1, 44_100)));

    {
        let mut s = seq.lock().unwrap();
        let idx = s.add_track(InstrumentKind::Piano, note.midi());
        s.tracks[idx].engine.fx.reverb.set_wet_mix(0.0);
        s.tracks[idx].engine.fx.delay.set_wet_mix(0.0);
    }

    let stream = build_stream(Arc::clone(&seq))?;
    stream.play()?;
    std::thread::sleep(std::time::Duration::from_millis(100));

    println!("Playing {} ({:.3} Hz) for 2 seconds...", note.name(), note.freq());

    {
        let mut s = seq.lock().unwrap();
        play_sound(note, 0.8, &mut s);
    }

    std::thread::sleep(std::time::Duration::from_millis(2000));

    {
        let mut s = seq.lock().unwrap();
        stop_sound(note, &mut s);
    }

    std::thread::sleep(std::time::Duration::from_millis(500));

    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

// Takes a note name string. Returns the matching PianoNote or None.
fn parse_note(name: &str) -> Option<PianoNote>
{
    match name
    {
        "A0"  => Some(PianoNote::A0),  "As0" | "Bb0" => Some(PianoNote::As0),
        "B0"  => Some(PianoNote::B0),
        "C1"  => Some(PianoNote::C1),  "Cs1" | "Db1" => Some(PianoNote::Cs1),
        "D1"  => Some(PianoNote::D1),  "Ds1" | "Eb1" => Some(PianoNote::Ds1),
        "E1"  => Some(PianoNote::E1),  "F1"  => Some(PianoNote::F1),
        "Fs1" | "Gb1" => Some(PianoNote::Fs1), "G1" => Some(PianoNote::G1),
        "Gs1" | "Ab1" => Some(PianoNote::Gs1), "A1" => Some(PianoNote::A1),
        "As1" | "Bb1" => Some(PianoNote::As1), "B1" => Some(PianoNote::B1),
        "C2"  => Some(PianoNote::C2),  "Cs2" | "Db2" => Some(PianoNote::Cs2),
        "D2"  => Some(PianoNote::D2),  "Ds2" | "Eb2" => Some(PianoNote::Ds2),
        "E2"  => Some(PianoNote::E2),  "F2"  => Some(PianoNote::F2),
        "Fs2" | "Gb2" => Some(PianoNote::Fs2), "G2" => Some(PianoNote::G2),
        "Gs2" | "Ab2" => Some(PianoNote::Gs2), "A2" => Some(PianoNote::A2),
        "As2" | "Bb2" => Some(PianoNote::As2), "B2" => Some(PianoNote::B2),
        "C3"  => Some(PianoNote::C3),  "Cs3" | "Db3" => Some(PianoNote::Cs3),
        "D3"  => Some(PianoNote::D3),  "Ds3" | "Eb3" => Some(PianoNote::Ds3),
        "E3"  => Some(PianoNote::E3),  "F3"  => Some(PianoNote::F3),
        "Fs3" | "Gb3" => Some(PianoNote::Fs3), "G3" => Some(PianoNote::G3),
        "Gs3" | "Ab3" => Some(PianoNote::Gs3), "A3" => Some(PianoNote::A3),
        "As3" | "Bb3" => Some(PianoNote::As3), "B3" => Some(PianoNote::B3),
        "C4"  => Some(PianoNote::C4),  "Cs4" | "Db4" => Some(PianoNote::Cs4),
        "D4"  => Some(PianoNote::D4),  "Ds4" | "Eb4" => Some(PianoNote::Ds4),
        "E4"  => Some(PianoNote::E4),  "F4"  => Some(PianoNote::F4),
        "Fs4" | "Gb4" => Some(PianoNote::Fs4), "G4" => Some(PianoNote::G4),
        "Gs4" | "Ab4" => Some(PianoNote::Gs4), "A4" => Some(PianoNote::A4),
        "As4" | "Bb4" => Some(PianoNote::As4), "B4" => Some(PianoNote::B4),
        "C5"  => Some(PianoNote::C5),  "Cs5" | "Db5" => Some(PianoNote::Cs5),
        "D5"  => Some(PianoNote::D5),  "Ds5" | "Eb5" => Some(PianoNote::Ds5),
        "E5"  => Some(PianoNote::E5),  "F5"  => Some(PianoNote::F5),
        "Fs5" | "Gb5" => Some(PianoNote::Fs5), "G5" => Some(PianoNote::G5),
        "Gs5" | "Ab5" => Some(PianoNote::Gs5), "A5" => Some(PianoNote::A5),
        "As5" | "Bb5" => Some(PianoNote::As5), "B5" => Some(PianoNote::B5),
        "C6"  => Some(PianoNote::C6),  "Cs6" | "Db6" => Some(PianoNote::Cs6),
        "D6"  => Some(PianoNote::D6),  "Ds6" | "Eb6" => Some(PianoNote::Ds6),
        "E6"  => Some(PianoNote::E6),  "F6"  => Some(PianoNote::F6),
        "Fs6" | "Gb6" => Some(PianoNote::Fs6), "G6" => Some(PianoNote::G6),
        "Gs6" | "Ab6" => Some(PianoNote::Gs6), "A6" => Some(PianoNote::A6),
        "As6" | "Bb6" => Some(PianoNote::As6), "B6" => Some(PianoNote::B6),
        "C7"  => Some(PianoNote::C7),  "Cs7" | "Db7" => Some(PianoNote::Cs7),
        "D7"  => Some(PianoNote::D7),  "Ds7" | "Eb7" => Some(PianoNote::Ds7),
        "E7"  => Some(PianoNote::E7),  "F7"  => Some(PianoNote::F7),
        "Fs7" | "Gb7" => Some(PianoNote::Fs7), "G7" => Some(PianoNote::G7),
        "Gs7" | "Ab7" => Some(PianoNote::Gs7), "A7" => Some(PianoNote::A7),
        "As7" | "Bb7" => Some(PianoNote::As7), "B7" => Some(PianoNote::B7),
        "C8"  => Some(PianoNote::C8),
        _ => None,
    }
}

// Builds the cpal audio stream with a shared sequencer behind a Mutex.
// The sequencer owns the Piano track — next_sample() drives the audio.
fn build_stream(seq: Arc<Mutex<StepSequencer>>) -> Result<cpal::Stream>
{
    let host   = cpal::default_host();
    let device = host
        .default_output_device()
        .ok_or_else(|| anyhow!("No output device found"))?;

    let config = StreamConfig
    {
        channels:    2,
        sample_rate: SampleRate(44_100),
        buffer_size: BufferSize::Fixed(1024),
    };

    let stream = device.build_output_stream(
        &config,
        move |output: &mut [f32], _: &cpal::OutputCallbackInfo|
        {
            let mut s = seq.lock().unwrap();
            for frame in output.chunks_mut(2)
            {
                // next_sample() sums all tracks inside the sequencer
                let sample = s.next_sample();
                for ch in frame.iter_mut()
                {
                    *ch = sample;
                }
            }
        },
        |err| eprintln!("[audio] Stream error: {err}"),
        None,
    )?;

    Ok(stream)
}

