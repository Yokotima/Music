/// audio/audio_engine.rs
///
/// "Happy Birthday" — Simple melody demo, ~10 seconds
/// Key: C major | BPM: 120
/// One instrument, melody only, no chords, no reverb issues.

use anyhow::{anyhow, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, SampleRate, StreamConfig};

use super::instruments::{InstrumentEngine, InstrumentKind};

pub const SAMPLE_RATE: u32 = 44_100;
pub const MAX_BUFFER_SIZE: u32 = 1024;

const BPM: f32 = 120.0;
const BEAT: u32 = (SAMPLE_RATE as f32 * 60.0 / BPM) as u32; // 1 beat = 22050 samples

// beat x100 → samples
fn s(beat_x100: u32) -> u32 {
    (beat_x100 as u64 * BEAT as u64 / 100) as u32
}

pub struct AudioEngine {
    _stream: cpal::Stream,
}

impl AudioEngine {
    pub fn start() -> Result<Self> {
        let host   = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| anyhow!("No output audio device found"))?;

        println!("[audio] Device : {}", device.name().unwrap_or("unknown".into()));

        let config = StreamConfig {
            channels:    2,
            sample_rate: SampleRate(SAMPLE_RATE),
            buffer_size: BufferSize::Fixed(MAX_BUFFER_SIZE),
        };

        println!(
            "[audio] Config : {} Hz | {} ch | buffer {} samples ({:.1} ms)",
            SAMPLE_RATE, config.channels, MAX_BUFFER_SIZE,
            MAX_BUFFER_SIZE as f32 / SAMPLE_RATE as f32 * 1000.0
        );

        // ══════════════════════════════════════════════════════════════
        // HAPPY BIRTHDAY — melody only
        //
        // C major, 120 BPM
        // (pitch, start x100, duration x100)
        // 100 = quarter note | 50 = eighth | 150 = dotted quarter | 200 = half
        //
        // MIDI notes:
        //   C4=60  D4=62  E4=64  F4=65  G4=67  A4=69  Bb4=70  B4=71  C5=72
        // ══════════════════════════════════════════════════════════════
        let melody: Vec<(u8, u32, u32)> = vec![
            // "Hap-py birth-day to you"
            (60,   0,  75), // C4  Hap-
            (60,  75,  25), // C4  -py
            (62, 100, 100), // D4  birth-
            (60, 200, 100), // C4  -day
            (65, 300, 100), // F4  to
            (64, 400, 200), // E4  you

            // "Hap-py birth-day to you"
            (60, 600,  75), // C4
            (60, 675,  25), // C4
            (62, 700, 100), // D4
            (60, 800, 100), // C4
            (67, 900, 100), // G4
            (65,1000, 200), // F4

            // "Hap-py birth-day dear [name]"
            (60,1200,  75), // C4
            (60,1275,  25), // C4
            (72,1300, 100), // C5  — jump up for emotion
            (69,1400, 100), // A4
            (65,1500, 100), // F4
            (64,1600, 100), // E4
            (62,1700, 200), // D4

            // "Hap-py birth-day to you"
            (70,1900,  75), // Bb4
            (70,1975,  25), // Bb4
            (69,2000, 100), // A4
            (65,2100, 100), // F4
            (67,2200, 100), // G4
            (65,2400, 200), // F4  — final note held
        ];

        // Build event timeline: (sample, note, is_on)
        let mut timeline: Vec<(u32, u8, bool)> = Vec::new();
        for &(pitch, start, dur) in &melody {
            timeline.push((s(start),       pitch, true));
            timeline.push((s(start + dur), pitch, false));
        }
        timeline.sort_by_key(|e| e.0);

        // Total: 2600 x100 beats = 26 quarter notes at 120 BPM ≈ 13 seconds
        let total_samples = s(2800);

        let channels       = config.channels as usize;
        let mut sample_ctr = 0u32;
        let mut cursor     = 0usize;

        // Piano — simple, clean, no effects overhead
        let mut piano = InstrumentEngine::new(InstrumentKind::Piano, SAMPLE_RATE);
        piano.fx.reverb.set_wet_mix(0.0); // dry — no reverb cost at all
        piano.fx.delay.set_wet_mix(0.0);  // no delay either

        println!("[audio] Playing: Happy Birthday (~13s then loops)");

        let stream = device.build_output_stream(
            &config,
            move |output: &mut [f32], _: &cpal::OutputCallbackInfo| {
                for frame in output.chunks_mut(channels) {
                    // Past the end — output silence, play once only
                    if sample_ctr >= total_samples {
                        for ch in frame.iter_mut() { *ch = 0.0; }
                        continue;
                    }

                    // Dispatch events
                    while cursor < timeline.len() && timeline[cursor].0 <= sample_ctr {
                        let (_, note, is_on) = timeline[cursor];
                        if is_on { piano.pool.note_on(note, 0.8); }
                        else     { piano.pool.note_off(note); }
                        cursor += 1;
                    }

                    let out = piano.next_sample();
                    for ch in frame.iter_mut() {
                        *ch = out;
                    }

                    sample_ctr += 1;
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
