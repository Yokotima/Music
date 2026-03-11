/// audio/audio_engine.rs
///
/// Sweden by C418 — Piano only, section 1:00 → 1:25
/// All notes use PianoNote enum from piano.rs — no raw MIDI numbers.

use anyhow::{anyhow, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, SampleRate, StreamConfig};

use crate::sequencer::sequencer::StepSequencer;
use super::instruments::InstrumentKind;
use super::piano::PianoNote;

pub const SAMPLE_RATE: u32     = 44_100;
pub const MAX_BUFFER_SIZE: u32 = 1024;

pub struct AudioEngine
{
    _stream: cpal::Stream,
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

        // 95 BPM, 32 steps — 1 step = 1 eighth note, 1 bar = 6 steps (3/4)
        let mut seq = StepSequencer::new(95.0, 32, SAMPLE_RATE);

        // Shorthand: convert PianoNote to midi u8
        let n = |note: PianoNote| note.midi();

        // ── Track 0: Melody (right hand) ──────────────────────────────────
        seq.add_track(InstrumentKind::Piano, n(PianoNote::Fs4));

        // Bar 1 — F#m
        seq.set_step_note(0,  0, n(PianoNote::Fs4));
        seq.set_step_note(0,  2, n(PianoNote::E4));
        seq.set_step_note(0,  4, n(PianoNote::D5));
        seq.set_step_note(0,  5, n(PianoNote::Cs5));

        // Bar 2 — Bm
        seq.set_step_note(0,  6, n(PianoNote::B4));
        seq.set_step_note(0,  8, n(PianoNote::A4));
        seq.set_step_note(0, 10, n(PianoNote::Gs4));

        // Bar 3 — A
        seq.set_step_note(0, 12, n(PianoNote::Fs4));
        seq.set_step_note(0, 14, n(PianoNote::E4));
        seq.set_step_note(0, 16, n(PianoNote::Cs5));
        seq.set_step_note(0, 17, n(PianoNote::B4));

        // Bar 4 — E
        seq.set_step_note(0, 18, n(PianoNote::A4));
        seq.set_step_note(0, 22, n(PianoNote::Fs4));

        // Bar 5 — D
        seq.set_step_note(0, 24, n(PianoNote::Gs4));
        seq.set_step_note(0, 26, n(PianoNote::Fs4));
        seq.set_step_note(0, 28, n(PianoNote::E4));
        seq.set_step_note(0, 29, n(PianoNote::D4));

        // Bar 6 — resolve
        seq.set_step_note(0, 30, n(PianoNote::Cs4));
        seq.set_step_note(0, 31, n(PianoNote::B3));

        // ── Track 1: Arpeggios (left hand) ────────────────────────────────
        // Pattern per bar: root - 3rd - 5th - 3rd - root - 3rd
        seq.add_track(InstrumentKind::Piano, n(PianoNote::Fs2));

        // Bar 1 — F#m  (Fs2 A2 Cs3)
        seq.set_step_note(1,  0, n(PianoNote::Fs2));
        seq.set_step_note(1,  1, n(PianoNote::A2));
        seq.set_step_note(1,  2, n(PianoNote::Cs3));
        seq.set_step_note(1,  3, n(PianoNote::A2));
        seq.set_step_note(1,  4, n(PianoNote::Fs2));
        seq.set_step_note(1,  5, n(PianoNote::A2));

        // Bar 2 — Bm  (B1 D2 Fs2)
        seq.set_step_note(1,  6, n(PianoNote::B1));
        seq.set_step_note(1,  7, n(PianoNote::D2));
        seq.set_step_note(1,  8, n(PianoNote::Fs2));
        seq.set_step_note(1,  9, n(PianoNote::D2));
        seq.set_step_note(1, 10, n(PianoNote::B1));
        seq.set_step_note(1, 11, n(PianoNote::D2));

        // Bar 3 — A  (A1 Cs2 E2)
        seq.set_step_note(1, 12, n(PianoNote::A1));
        seq.set_step_note(1, 13, n(PianoNote::Cs2));
        seq.set_step_note(1, 14, n(PianoNote::E2));
        seq.set_step_note(1, 15, n(PianoNote::Cs2));
        seq.set_step_note(1, 16, n(PianoNote::A1));
        seq.set_step_note(1, 17, n(PianoNote::Cs2));

        // Bar 4 — E  (E2 Gs2 B2)
        seq.set_step_note(1, 18, n(PianoNote::E2));
        seq.set_step_note(1, 19, n(PianoNote::Gs2));
        seq.set_step_note(1, 20, n(PianoNote::B2));
        seq.set_step_note(1, 21, n(PianoNote::Gs2));
        seq.set_step_note(1, 22, n(PianoNote::E2));
        seq.set_step_note(1, 23, n(PianoNote::Gs2));

        // Bar 5 — D  (D2 Fs2 A2)
        seq.set_step_note(1, 24, n(PianoNote::D2));
        seq.set_step_note(1, 25, n(PianoNote::Fs2));
        seq.set_step_note(1, 26, n(PianoNote::A2));
        seq.set_step_note(1, 27, n(PianoNote::Fs2));
        seq.set_step_note(1, 28, n(PianoNote::D2));
        seq.set_step_note(1, 29, n(PianoNote::Fs2));

        // Bar 6 — F#m resolve
        seq.set_step_note(1, 30, n(PianoNote::Fs2));
        seq.set_step_note(1, 31, n(PianoNote::A2));

        // ── Velocities ────────────────────────────────────────────────────
        seq.tracks[0].default_velocity = 0.82; // melody
        seq.tracks[1].default_velocity = 0.42; // arpeggios

        seq.play();

        println!("[audio] Sweden (1:00-1:25) — Piano | F# minor | 95 BPM");
        println!("[audio] 2 tracks | 32 steps | loops");

        let channels = config.channels as usize;

        let stream = device.build_output_stream(
            &config,
            move |output: &mut [f32], _: &cpal::OutputCallbackInfo|
            {
                for frame in output.chunks_mut(channels)
                {
                    let out = seq.next_sample();
                    for ch in frame.iter_mut()
                    {
                        *ch = out;
                    }
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
