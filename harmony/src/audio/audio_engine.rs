/// audio/audio_engine.rs
///
/// Step 1 of HarmonyStudio DSP engine.
///
/// Responsibilities:
///   - Open a cpal output stream at 44100 Hz, stereo, f32 samples
///   - Confirm buffer size <= 512 samples (CDC constraint: latency < 20ms)
///   - Feed a 440 Hz sine wave (A4) with no glitches into the output
///
/// Architecture note (from CDC §4.2):
///   The audio callback runs in a dedicated real-time thread managed by cpal.
///   NO heap allocation, NO mutex, NO blocking calls are allowed inside the
///   callback. All shared state will later be passed via ringbuf SPSC buffers.

use anyhow::{anyhow, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, SampleRate, StreamConfig};
use std::f32::consts::TAU; // TAU = 2 * PI

// ─────────────────────────────────────────────────────────────────────────────
// Constants
// ─────────────────────────────────────────────────────────────────────────────

/// Target sample rate (Hz). Standard for audio production.
pub const SAMPLE_RATE: u32 = 44_100;

/// Maximum buffer size in samples (CDC §4.3: must be <= 512 for < 20ms latency).
/// At 44100 Hz: 512 samples ≈ 11.6 ms of latency.
pub const MAX_BUFFER_SIZE: u32 = 512;

/// Test tone frequency in Hz. A4 = 440 Hz, easy to verify by ear.
const TEST_FREQUENCY_HZ: f32 = 440.0;

/// Test tone amplitude (0.0 to 1.0). Keep moderate to protect your ears.
const TEST_AMPLITUDE: f32 = 0.3;

// ─────────────────────────────────────────────────────────────────────────────
// SineOscillator
// ─────────────────────────────────────────────────────────────────────────────

/// A simple sine wave oscillator.
///
/// State: a single `phase` in [0.0, 1.0).
/// Each sample advances the phase by `frequency / sample_rate`.
///
/// This will be replaced in Step 2 by a full PolyBLEP oscillator
/// supporting saw, square, and triangle waveforms.
struct SineOscillator {
    phase: f32,
    phase_increment: f32,
}

impl SineOscillator {
    fn new(frequency_hz: f32, sample_rate: u32) -> Self {
        Self {
            phase: 0.0,
            phase_increment: frequency_hz / sample_rate as f32,
        }
    }

    /// Compute the next sample and advance the phase.
    ///
    /// `#[inline(always)]` helps LLVM vectorize loops that call this.
    /// Phase wraps with subtraction (not %) — faster and avoids float drift.
    #[inline(always)]
    fn next_sample(&mut self) -> f32 {
        let sample = (self.phase * TAU).sin();

        self.phase += self.phase_increment;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        sample
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// AudioEngine
// ─────────────────────────────────────────────────────────────────────────────

/// The main audio engine handle.
///
/// Holds the cpal stream alive via RAII — dropping this struct stops the stream.
///
/// In later steps this will also hold:
///   - The voice pool (polyphony, Step 4)
///   - A ringbuf receiver for NoteOn/NoteOff events from the sequencer (Step 8)
///   - Instrument and effect parameter handles
pub struct AudioEngine {
    _stream: cpal::Stream,
}

impl AudioEngine {
    /// Initialize and start the audio engine.
    ///
    /// 1. Find the default output device
    /// 2. Negotiate config: 44100 Hz, stereo f32, buffer ≤ 512 samples
    /// 3. Spawn the cpal real-time callback
    /// 4. Return the handle (stream runs until handle is dropped)
    pub fn start() -> Result<Self> {
        // ── 1. Host & device ───────────────────────────────────────────────
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| anyhow!("No output audio device found"))?;

        println!("[audio] Device : {}", device.name().unwrap_or("unknown".into()));

        // ── 2. Stream config ───────────────────────────────────────────────
        let config = StreamConfig {
            channels: 2,
            sample_rate: SampleRate(SAMPLE_RATE),
            buffer_size: BufferSize::Fixed(MAX_BUFFER_SIZE),
        };

        println!(
            "[audio] Config : {} Hz | {} ch | buffer {} samples ({:.1} ms)",
            SAMPLE_RATE,
            config.channels,
            MAX_BUFFER_SIZE,
            MAX_BUFFER_SIZE as f32 / SAMPLE_RATE as f32 * 1000.0
        );

        // ── 3. Callback state (moved into the closure) ─────────────────────
        // Everything the callback needs must be owned by it.
        // No shared refs, no Mutex — this closure runs on the RT thread.
        let mut oscillator = SineOscillator::new(TEST_FREQUENCY_HZ, SAMPLE_RATE);
        let channels = config.channels as usize;

        // ── 4. Build & start stream ────────────────────────────────────────
        let stream = device.build_output_stream(
            &config,
            // DATA CALLBACK — real-time thread, called by cpal repeatedly
            // output is interleaved: [L0, R0, L1, R1, ...]
            move |output: &mut [f32], _: &cpal::OutputCallbackInfo| {
                for frame in output.chunks_mut(channels) {
                    let sample = oscillator.next_sample() * TEST_AMPLITUDE;
                    for ch in frame.iter_mut() {
                        *ch = sample;
                    }
                }
            },
            // ERROR CALLBACK
            |err| eprintln!("[audio] Stream error: {err}"),
            None,
        )?;

        stream.play()?;
        // Small delay to let the PulseAudio/WSLg server stabilize before
        // the callback starts firing. Harmless on native Linux, fixes WSL startup glitches.
        std::thread::sleep(std::time::Duration::from_millis(300));
        println!("[audio] Stream started ");

        Ok(Self { _stream: stream })
    }
}
