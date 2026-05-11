#![allow(dead_code)]
/// ## The aliasing problem
///
/// Naive waveforms like sawtooth or square have abrupt discontinuities.
/// These sharp jumps contain energy at ALL frequencies, including above
/// the Nyquist limit (sample_rate / 2). Those high-frequency components
/// "fold back" into the audible range as metallic, harsh artifacts.
///
/// ## The PolyBLEP solution
///
/// PolyBLEP (Polynomial Band-Limited Step) smooths each discontinuity
/// by adding a small correction polynomial right around the jump point.
/// The correction is computed locally — only 1-2 samples around the
/// discontinuity are affected — making it extremely cheap to compute.
///
/// ## Waveforms implemented
///
///   Sine     — no discontinuity, no correction needed
///   Sawtooth — one discontinuity per cycle  (phase = 1.0 → 0.0)
///   Square   — two discontinuities per cycle (phase = 0.0 and 0.5)
///   Triangle — derived by integrating a square wave (no extra correction)

use std::f32::consts::TAU;

//==========Waveform enum==========
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Waveform
{
    Sine,
    Sawtooth,
    Square,
    Triangle,
    // Physical model — piano string simulation.
    // Uses a delay line with a low-pass filter in feedback to simulate
    // a hammered string. Much more realistic than additive synthesis.
    KarplusStrong,
}

//==========PolyBLEP correction function==========
#[inline(always)]
fn poly_blep(phase: f32, dt: f32) -> f32
{
    if phase < dt
    {
        let t = phase / dt;
        2.0 * t - t * t - 1.0
    }
    else if phase > 1.0 - dt
    {
        let t = (phase - 1.0) / dt;
        t * t + 2.0 * t + 1.0
    }
    else
    {
        0.0
    }
}

//==========Oscillator==========
/// Triple-oscillator unison generator.
/// Produces richer sound by slightly detuning 3 oscillators.
pub struct Oscillator
{
    phase1:f32,
    phase2:f32,
    phase3:f32,

    phase_increment:f32,
    detune:f32,

    waveform:Waveform,
    last_triangle:f32,
    sample_rate:u32,

    // Karplus-Strong state
    // delay line stores one period of the string vibration
    ks_buffer:    Vec<f32>,
    ks_pos:       usize,
    ks_prev:      f32,    // previous output — used by the LP filter in feedback
    ks_ready:     bool,   // true once the buffer has been seeded with noise
}

impl Oscillator
{
    // Takes frequency, sample rate, waveform
    pub fn new(frequency_hz: f32, sample_rate: u32, waveform: Waveform) -> Self
    {
        let max_period = (sample_rate as f32 / 20.0).ceil() as usize + 4; // A0 = 27.5 Hz
        Self
        {
            phase1:0.0,
            phase2:0.0,
            phase3:0.0,

            phase_increment:frequency_hz / sample_rate as f32,
            detune:0.002,

            waveform,
            last_triangle:0.0,
            sample_rate,

            ks_buffer: vec![0.0; max_period],
            ks_pos:    0,
            ks_prev:   0.0,
            ks_ready:  false,
        }
    }

    // Update pitch
    pub fn set_frequency(&mut self, frequency_hz: f32)
    {
        self.phase_increment = frequency_hz / self.sample_rate as f32;

        // Re-seed the KS buffer for the new pitch so the note starts cleanly
        if self.waveform == Waveform::KarplusStrong
        {
            self.ks_seed(frequency_hz);
        }
    }

    // Change waveform
    pub fn set_waveform(&mut self, waveform: Waveform)
    {
        self.waveform = waveform;
        self.last_triangle = 0.0;
    }

    // Seeds the Karplus-Strong buffer with band-limited noise shaped to
    // simulate a piano hammer strike.
    // Called on every note_on via set_frequency().
    fn ks_seed(&mut self, frequency_hz: f32)
    {
        let period = (self.sample_rate as f32 / frequency_hz).round() as usize;
        let period = period.max(2).min(self.ks_buffer.len());

        // Fill the delay line with filtered white noise (hammer impulse).
        // We use a simple one-pole high-pass to remove DC and shape the
        // attack so it sounds like a hammer hit rather than a pluck.
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut prev = 0.0f32;
        for i in 0..period
        {
            // Deterministic pseudo-noise seeded by position
            let mut h = DefaultHasher::new();
            (i as u64 ^ 0xDEAD_BEEF).hash(&mut h);
            let raw = (h.finish() as i64) as f32 / i64::MAX as f32; // -1..1

            // One-pole high-pass: removes DC from the noise burst
            let hp = raw - prev * 0.9;
            prev = raw;
            self.ks_buffer[i] = hp * 0.5;
        }

        // Taper the end of the buffer to avoid a click at loop point
        let taper = (period / 8).max(1);
        for i in (period - taper)..period
        {
            let t = (period - i) as f32 / taper as f32;
            self.ks_buffer[i] *= t;
        }

        self.ks_pos   = 0;
        self.ks_prev  = 0.0;
        self.ks_ready = true;
    }

    //==========Main sample generation==========
    #[inline(always)]
    pub fn next_sample(&mut self) -> (f32, f32) { // Return stereo tuple
        if self.waveform == Waveform::KarplusStrong {
            let mono = self.ks_next_sample();
            return (mono, mono); // KS remains mono for core stability
        }

        let dt = self.phase_increment;
        let s1 = self.sample_from_phase(self.phase1);
        let s2 = self.sample_from_phase(self.phase2);
        let s3 = self.sample_from_phase(self.phase3);

        // Spread them in the stereo field:
        // Oscillator 1: Center
        // Oscillator 2: Hard Left (+ detune)
        // Oscillator 3: Hard Right (- detune)
        let left = (s1 * 0.5) + (s2 * 0.7);
        let right = (s1 * 0.5) + (s3 * 0.7);

        // Advance phases
        self.phase1 += dt;
        self.phase2 += dt * (1.0 + self.detune);
        self.phase3 += dt * (1.0 - self.detune);

        // Phase wrapping logic
        if self.phase1 >= 1.0 { self.phase1 -= 1.0; }
        if self.phase2 >= 1.0 { self.phase2 -= 1.0; }
        if self.phase3 >= 1.0 { self.phase3 -= 1.0; }

        (left * 0.6, right * 0.6)
    }
    // Karplus-Strong per-sample step.
    //
    // Algorithm:
    //   1. Read the current sample from the delay line
    //   2. Apply a one-pole low-pass filter in the feedback path:
    //        filtered = (current + prev) * 0.5 * decay
    //      This damps high frequencies faster than low — exactly what a
    //      real piano string does (high partials decay quicker).
    //   3. Write the filtered value back to the delay line
    //   4. Output the current (pre-filter) value
    //
    // The decay constant controls how fast the string loses energy.
    // A value close to 1.0 = long sustain (bass strings).
    // A value like 0.996 = shorter decay (treble strings).
    //
    // We scale decay with frequency: bass notes sustain longer,
    // treble notes decay faster — exactly like a real piano.
    #[inline(always)]
    fn ks_next_sample(&mut self) -> f32
    {
        if !self.ks_ready
        {
            let freq = self.phase_increment * self.sample_rate as f32;
            self.ks_seed(freq.max(20.0));
        }

        let period = (self.sample_rate as f32 * self.phase_increment.recip()).round() as usize;
        let period = period.max(2).min(self.ks_buffer.len());

        let out = self.ks_buffer[self.ks_pos];

        // One-pole LP filter in feedback: averages current with previous
        // The decay factor: higher frequency → faster decay (0.990 to 0.9995)
        // Maps freq 27 Hz (A0) → 0.9995, freq 4186 Hz (C8) → 0.990
        let freq_hz = self.phase_increment * self.sample_rate as f32;
        let decay   = (1.0 - freq_hz / 80_000.0).clamp(0.990, 0.9995);

        let filtered = (out + self.ks_prev) * 0.5 * decay;
        self.ks_prev = out;

        self.ks_buffer[self.ks_pos] = filtered;
        self.ks_pos = (self.ks_pos + 1) % period;

        out
    }

    //==========Waveform generator==========
    #[inline(always)]
    fn sample_from_phase(&mut self, phase: f32) -> f32
    {
        match self.waveform
        {
            Waveform::Sine =>
            {
                (phase * TAU).sin()
            }

            Waveform::Sawtooth =>
            {
                let naive = 2.0 * phase - 1.0;
                naive - poly_blep(phase, self.phase_increment)
            }

            Waveform::Square =>
            {
                let naive = if phase < 0.5 { 1.0 } else { -1.0 };

                let correction_rise = poly_blep(phase, self.phase_increment);

                let mut phase_shifted = phase + 0.5;
                if phase_shifted >= 1.0
                {
                    phase_shifted -= 1.0;
                }

                let correction_fall = poly_blep(phase_shifted, self.phase_increment);

                naive + correction_rise - correction_fall
            }

            Waveform::Triangle =>
            {
                let square = if phase < 0.5 { 1.0 } else { -1.0 };
                let dt = self.phase_increment;

                let tri = dt * square + (1.0 - dt) * self.last_triangle;

                let output = tri * (4.0 * dt).recip().min(1.0);

                self.last_triangle = tri;

                output
            }

            // KarplusStrong never reaches sample_from_phase —
            // next_sample() returns early via ks_next_sample().
            // This arm exists only to satisfy the exhaustive match.
            Waveform::KarplusStrong => 0.0,
        }
    }
}
