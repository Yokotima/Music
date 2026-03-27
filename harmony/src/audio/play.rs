/// This is the ONLY file the rest of the app needs to import for playing sounds.
/// All cpal setup, sequencer wiring, and thread management is hidden inside.
///
/// ┌──────────────────────────────────────────────────────────────┐
/// │  App layer (UI, game logic, tests…)                          │
/// │                                                              │
/// │    play(MidiNote::C4, InstrumentKind::Piano, 2.0);           │
/// │    play_chord(&[C4, E4, G4], InstrumentKind::Piano, 1.5);    │
/// │    play_melody(&[C4,D4,E4], InstrumentKind::Flute, 0.4, 0.05)│
/// └──────────────────────────┬───────────────────────────────────┘
///                            │  (this file)
/// ┌──────────────────────────▼───────────────────────────────────┐
/// │  play.rs  — builds stream, sequencer, track; drives cpal     │
/// └──────────────────────────┬───────────────────────────────────┘
///                            │
/// ┌──────────────────────────▼───────────────────────────────────┐
/// │  instruments.rs  /  sequencer.rs  /  voice.rs  /  …          │
/// │  (engine internals — app never touches these directly)        │
/// └──────────────────────────────────────────────────────────────┘

use anyhow::Result;
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, SampleRate, StreamConfig};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::sequencer::sequencer::StepSequencer;
use super::instruments::InstrumentKind;
use super::note::MidiNote;

const SAMPLE_RATE: u32 = 44_100;
const BUFFER_SIZE: u32 = 1_024;
const DEFAULT_VELOCITY: f32 = 0.8;

// play one note and block until it finishes
pub fn play(note: MidiNote, instrument: InstrumentKind, duration_secs: f32)
{
    if let Err(e) = play_inner(&[note], instrument, duration_secs)
    {
        eprintln!("[play] Error: {e}");
    }
}

// play multiple notes simultaneously and block until done
pub fn play_chord(notes: &[MidiNote], instrument: InstrumentKind, duration_secs: f32)
{
    if notes.is_empty() { return; }
    if let Err(e) = play_inner(notes, instrument, duration_secs)
    {
        eprintln!("[play_chord] Error: {e}");
    }
}

// play notes one after another and block until done
pub fn play_melody(notes: &[MidiNote], instrument: InstrumentKind, note_duration: f32, gap_secs: f32)
{
    for &note in notes
    {
        play(note, instrument, note_duration);
        if gap_secs > 0.0
        {
            std::thread::sleep(Duration::from_secs_f32(gap_secs));
        }
    }
}

pub fn play_async(note: MidiNote, instrument: InstrumentKind, duration_secs: f32) -> std::thread::JoinHandle<()>
{
    std::thread::spawn(move || play(note, instrument, duration_secs))
}

//==========DO NOT USE EVERYTHING AFTER TO THIS WARNING==========\\

// Shared implementation for play() and play_chord().
// Builds a cpal stream, fires note_on for every note, sleeps, then note_off.
fn play_inner(notes: &[MidiNote], instrument: InstrumentKind, duration_secs: f32) -> Result<()>
{
    let seq = Arc::new(Mutex::new(StepSequencer::new(120.0, 1, SAMPLE_RATE)));
    {
        let mut s = seq.lock().unwrap();
        let default_midi = notes.first().map(|n| n.midi()).unwrap_or(60);
        let idx = s.add_track(instrument, default_midi);

        // Silence effects for clean isolated playback.
        s.tracks[idx].engine.fx.delay.set_wet_mix(0.0);
        s.tracks[idx].engine.fx.delay.set_feedback(0.0);
        s.tracks[idx].engine.fx.reverb.set_wet_mix(0.0);
    }

    let stream = build_stream(Arc::clone(&seq))?;
    stream.play()?;

    // Give the audio thread a moment to start
    std::thread::sleep(Duration::from_millis(15));

    {
        let mut s = seq.lock().unwrap();
        let idx = 0; // the track we just created is always index 0 here
        for note in notes
        {
            s.tracks[idx].engine.pool.note_on(note.midi(), DEFAULT_VELOCITY);
            println!("[play] ON  {} ({}) {:.1} Hz",
                note.name(), instrument.name(), note.freq());
        }
    }

    std::thread::sleep(Duration::from_secs_f32(duration_secs));

    {
        let mut s = seq.lock().unwrap();
        for note in notes
        {
            s.tracks[0].engine.pool.note_off(note.midi());
            println!("[play] OFF {}", note.name());
        }
    }

    std::thread::sleep(Duration::from_millis(350));

    drop(stream);
    Ok(())
}

// Builds and returns a cpal output stream backed by the given sequencer.
// The closure calls seq.next_sample() 44 100 times per second.
fn build_stream(seq: Arc<Mutex<StepSequencer>>) -> Result<cpal::Stream>
{
    let host = cpal::default_host();
    let device = host.default_output_device().ok_or_else(|| anyhow::anyhow!("No audio output device found"))?;

    let config = StreamConfig
    {
        channels: 2,
        sample_rate: SampleRate(SAMPLE_RATE),
        buffer_size: BufferSize::Fixed(BUFFER_SIZE),
    };

    let stream = device.build_output_stream(
        &config,
        move |output: &mut [f32], _: &cpal::OutputCallbackInfo|
        {
            let mut s = seq.lock().unwrap();
            for frame in output.chunks_mut(2)
            {
                let (l, r) = s.next_sample();

                if frame.len() >= 2 {
                    frame[0] = l;
                    frame[1] = r;
                }
            }
        },
        |err| eprintln!("[play] Stream error: {err}"),
        None,
    )?;

    Ok(stream)
}
