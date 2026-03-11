/// audio/audio_engine.rs — Sweden by C418, Full song, Piano only
/// Key: F# minor | BPM: 95 | Time: 3/4
/// Structure: A → B → B → C → B → D
/// All notes use PianoNote from piano.rs

use anyhow::{anyhow, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, SampleRate, StreamConfig};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use crate::sequencer::sequencer::StepSequencer;
use super::instruments::InstrumentKind;
use super::piano::PianoNote;

pub const SAMPLE_RATE: u32     = 44_100;
pub const MAX_BUFFER_SIZE: u32 = 1024;

fn n(note: PianoNote) -> u8 { note.midi() }

pub struct AudioEngine { _stream: cpal::Stream }

fn build_section_a(sr: u32) -> StepSequencer
{
    let mut seq = StepSequencer::new(95.0, 48, sr);
    seq.add_track(InstrumentKind::Piano, n(PianoNote::Fs4));
    seq.set_step_note(0, 18, n(PianoNote::Fs4));
    seq.set_step_note(0, 20, n(PianoNote::E4));
    seq.set_step_note(0, 22, n(PianoNote::Cs4));
    seq.set_step_note(0, 24, n(PianoNote::D4));
    seq.set_step_note(0, 26, n(PianoNote::Cs4));
    seq.set_step_note(0, 28, n(PianoNote::B3));
    seq.set_step_note(0, 30, n(PianoNote::A3));
    seq.set_step_note(0, 32, n(PianoNote::B3));
    seq.set_step_note(0, 34, n(PianoNote::Cs4));
    seq.set_step_note(0, 36, n(PianoNote::D4));
    seq.set_step_note(0, 40, n(PianoNote::Cs4));
    seq.set_step_note(0, 44, n(PianoNote::B3));
    seq.set_step_note(0, 46, n(PianoNote::Fs3));

    let chords: [(PianoNote, PianoNote, PianoNote); 8] = [
        (PianoNote::Fs2, PianoNote::A2,  PianoNote::Cs3),
        (PianoNote::Fs2, PianoNote::A2,  PianoNote::Cs3),
        (PianoNote::B1,  PianoNote::D2,  PianoNote::Fs2),
        (PianoNote::B1,  PianoNote::D2,  PianoNote::Fs2),
        (PianoNote::A1,  PianoNote::Cs2, PianoNote::E2),
        (PianoNote::A1,  PianoNote::Cs2, PianoNote::E2),
        (PianoNote::E2,  PianoNote::Gs2, PianoNote::B2),
        (PianoNote::E2,  PianoNote::Gs2, PianoNote::B2),
    ];
    seq.add_track(InstrumentKind::Piano, n(PianoNote::Fs2));
    for (bar, (r, t, f)) in chords.iter().enumerate() {
        let s = bar * 6;
        seq.set_step_note(1, s,     n(*r));
        seq.set_step_note(1, s + 1, n(*t));
        seq.set_step_note(1, s + 2, n(*f));
        seq.set_step_note(1, s + 3, n(*t));
        seq.set_step_note(1, s + 4, n(*r));
        seq.set_step_note(1, s + 5, n(*t));
    }
    seq.tracks[0].default_velocity = 0.60;
    seq.tracks[1].default_velocity = 0.35;
    seq.looping = false;
    seq.play();
    seq
}

fn build_section_b(sr: u32) -> StepSequencer
{
    let mut seq = StepSequencer::new(95.0, 48, sr);
    seq.add_track(InstrumentKind::Piano, n(PianoNote::Fs4));
    seq.set_step_note(0,  0, n(PianoNote::Fs4));
    seq.set_step_note(0,  2, n(PianoNote::E4));
    seq.set_step_note(0,  4, n(PianoNote::D5));
    seq.set_step_note(0,  5, n(PianoNote::Cs5));
    seq.set_step_note(0,  6, n(PianoNote::B4));
    seq.set_step_note(0,  8, n(PianoNote::A4));
    seq.set_step_note(0, 10, n(PianoNote::Gs4));
    seq.set_step_note(0, 12, n(PianoNote::Fs4));
    seq.set_step_note(0, 14, n(PianoNote::E4));
    seq.set_step_note(0, 16, n(PianoNote::Cs5));
    seq.set_step_note(0, 17, n(PianoNote::B4));
    seq.set_step_note(0, 18, n(PianoNote::A4));
    seq.set_step_note(0, 22, n(PianoNote::Fs4));
    seq.set_step_note(0, 24, n(PianoNote::Gs4));
    seq.set_step_note(0, 26, n(PianoNote::Fs4));
    seq.set_step_note(0, 28, n(PianoNote::E4));
    seq.set_step_note(0, 29, n(PianoNote::D4));
    seq.set_step_note(0, 30, n(PianoNote::Cs4));
    seq.set_step_note(0, 32, n(PianoNote::B3));
    seq.set_step_note(0, 34, n(PianoNote::A3));
    seq.set_step_note(0, 36, n(PianoNote::Fs4));
    seq.set_step_note(0, 38, n(PianoNote::E4));
    seq.set_step_note(0, 40, n(PianoNote::D4));
    seq.set_step_note(0, 42, n(PianoNote::Cs4));
    seq.set_step_note(0, 44, n(PianoNote::B3));
    seq.set_step_note(0, 46, n(PianoNote::Fs3));

    let chords: [(PianoNote, PianoNote, PianoNote); 8] = [
        (PianoNote::Fs2, PianoNote::A2,  PianoNote::Cs3),
        (PianoNote::B1,  PianoNote::D2,  PianoNote::Fs2),
        (PianoNote::A1,  PianoNote::Cs2, PianoNote::E2),
        (PianoNote::E2,  PianoNote::Gs2, PianoNote::B2),
        (PianoNote::D2,  PianoNote::Fs2, PianoNote::A2),
        (PianoNote::Cs2, PianoNote::E2,  PianoNote::Gs2),
        (PianoNote::B1,  PianoNote::D2,  PianoNote::Fs2),
        (PianoNote::Fs2, PianoNote::A2,  PianoNote::Cs3),
    ];
    seq.add_track(InstrumentKind::Piano, n(PianoNote::Fs2));
    for (bar, (r, t, f)) in chords.iter().enumerate() {
        let s = bar * 6;
        seq.set_step_note(1, s,     n(*r));
        seq.set_step_note(1, s + 1, n(*t));
        seq.set_step_note(1, s + 2, n(*f));
        seq.set_step_note(1, s + 3, n(*t));
        seq.set_step_note(1, s + 4, n(*r));
        seq.set_step_note(1, s + 5, n(*t));
    }
    seq.tracks[0].default_velocity = 0.82;
    seq.tracks[1].default_velocity = 0.42;
    seq.looping = false;
    seq.play();
    seq
}

fn build_section_c(sr: u32) -> StepSequencer
{
    let mut seq = StepSequencer::new(95.0, 48, sr);
    seq.add_track(InstrumentKind::Piano, n(PianoNote::Fs5));
    seq.set_step_note(0,  0, n(PianoNote::Fs5));
    seq.set_step_note(0,  2, n(PianoNote::E5));
    seq.set_step_note(0,  4, n(PianoNote::Cs5));
    seq.set_step_note(0,  6, n(PianoNote::B4));
    seq.set_step_note(0,  8, n(PianoNote::A4));
    seq.set_step_note(0, 10, n(PianoNote::Gs4));
    seq.set_step_note(0, 12, n(PianoNote::A4));
    seq.set_step_note(0, 14, n(PianoNote::B4));
    seq.set_step_note(0, 16, n(PianoNote::Cs5));
    seq.set_step_note(0, 18, n(PianoNote::D5));
    seq.set_step_note(0, 20, n(PianoNote::E5));
    seq.set_step_note(0, 22, n(PianoNote::Fs5));
    seq.set_step_note(0, 24, n(PianoNote::E5));
    seq.set_step_note(0, 26, n(PianoNote::D5));
    seq.set_step_note(0, 28, n(PianoNote::Cs5));
    seq.set_step_note(0, 30, n(PianoNote::B4));
    seq.set_step_note(0, 32, n(PianoNote::A4));
    seq.set_step_note(0, 34, n(PianoNote::Gs4));
    seq.set_step_note(0, 36, n(PianoNote::Fs4));
    seq.set_step_note(0, 38, n(PianoNote::E4));
    seq.set_step_note(0, 40, n(PianoNote::D4));
    seq.set_step_note(0, 44, n(PianoNote::Cs4));
    seq.set_step_note(0, 46, n(PianoNote::B3));

    seq.add_track(InstrumentKind::Piano, n(PianoNote::Fs2));
    let bass: [PianoNote; 8] = [
        PianoNote::Fs2, PianoNote::B1,  PianoNote::A1,  PianoNote::E2,
        PianoNote::D2,  PianoNote::Cs2, PianoNote::B1,  PianoNote::Fs2,
    ];
    for (bar, &root) in bass.iter().enumerate() {
        let s = bar * 6;
        seq.set_step_note(1, s,     n(root));
        seq.set_step_note(1, s + 3, n(root));
    }
    seq.tracks[0].default_velocity = 0.88;
    seq.tracks[1].default_velocity = 0.52;
    seq.looping = false;
    seq.play();
    seq
}

fn build_section_d(sr: u32) -> StepSequencer
{
    let mut seq = StepSequencer::new(95.0, 48, sr);
    seq.add_track(InstrumentKind::Piano, n(PianoNote::Fs4));
    seq.set_step_note(0,  0, n(PianoNote::Fs4));
    seq.set_step_note(0,  4, n(PianoNote::E4));
    seq.set_step_note(0,  8, n(PianoNote::Cs4));
    seq.set_step_note(0, 12, n(PianoNote::D4));
    seq.set_step_note(0, 18, n(PianoNote::Cs4));
    seq.set_step_note(0, 24, n(PianoNote::B3));
    seq.set_step_note(0, 30, n(PianoNote::A3));
    seq.set_step_note(0, 36, n(PianoNote::Fs3));
    seq.set_step_note(0, 42, n(PianoNote::E3));
    seq.set_step_note(0, 46, n(PianoNote::Fs3));

    seq.add_track(InstrumentKind::Piano, n(PianoNote::Fs2));
    for bar in 0..4 {
        let s = bar * 6;
        seq.set_step_note(1, s,     n(PianoNote::Fs2));
        seq.set_step_note(1, s + 3, n(PianoNote::Cs3));
    }
    for bar in 4..8 {
        let s = bar * 6;
        seq.set_step_note(1, s,     n(PianoNote::E2));
        seq.set_step_note(1, s + 3, n(PianoNote::B2));
    }
    seq.tracks[0].default_velocity = 0.55;
    seq.tracks[1].default_velocity = 0.28;
    seq.looping = false;
    seq.play();
    seq
}

impl AudioEngine
{
    pub fn start() -> Result<Self>
    {
        let host   = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| anyhow!("No output audio device found"))?;

        println!("[audio] Device : {}", device.name().unwrap_or("unknown".into()));

        let config = StreamConfig
        {
            channels:    2,
            sample_rate: SampleRate(SAMPLE_RATE),
            buffer_size: BufferSize::Fixed(MAX_BUFFER_SIZE),
        };

        println!(
            "[audio] Config : {} Hz | {} ch | buffer {} samples ({:.1} ms)",
            SAMPLE_RATE, config.channels, MAX_BUFFER_SIZE,
            MAX_BUFFER_SIZE as f32 / SAMPLE_RATE as f32 * 1000.0
        );

        // A → B → B → C → B → D
        let mut sections: Vec<StepSequencer> = vec![
            build_section_a(SAMPLE_RATE),
            build_section_b(SAMPLE_RATE),
            build_section_b(SAMPLE_RATE),
            build_section_c(SAMPLE_RATE),
            build_section_b(SAMPLE_RATE),
            build_section_d(SAMPLE_RATE),
        ];

        let section_idx    = Arc::new(AtomicUsize::new(0));
        let section_idx_cb = Arc::clone(&section_idx);
        let channels       = config.channels as usize;

        println!("[audio] Sweden — Full song | Piano | F# minor | 95 BPM");
        println!("[audio] Intro → Theme → Theme → Dev → Theme → Outro");
        println!("[audio] Section 1 / {}", sections.len());

        let stream = device.build_output_stream(
            &config,
            move |output: &mut [f32], _: &cpal::OutputCallbackInfo|
            {
                for frame in output.chunks_mut(channels)
                {
                    let idx = section_idx_cb.load(Ordering::Relaxed);

                    if idx < sections.len() && !sections[idx].playing
                    {
                        let next = idx + 1;
                        section_idx_cb.store(next, Ordering::Relaxed);
                        if next < sections.len() {
                            eprintln!("[audio] Section {} / {}", next + 1, sections.len());
                        } else {
                            eprintln!("[audio] Song finished.");
                        }
                    }

                    let out = if idx < sections.len()
                    {
                        sections[idx].next_sample()
                    }
                    else
                    {
                        0.0
                    };

                    for ch in frame.iter_mut() { *ch = out; }
                }
            },
            |err| eprintln!("[audio] Stream error: {err}"),
            None,
        )?;

        stream.play()?;
        std::thread::sleep(std::time::Duration::from_millis(100));
        println!("[audio] Stream started ✓ — press ENTER to stop");

        Ok(Self { _stream: stream })
    }
}
