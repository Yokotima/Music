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

// ─────────────────────────────────────────────────────────────────────────────
// Waveform enum
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Waveform {
    Sine,
    Sawtooth,
    Square,
    Triangle,
}

// ─────────────────────────────────────────────────────────────────────────────
// PolyBLEP correction function
// ─────────────────────────────────────────────────────────────────────────────

/// Compute the PolyBLEP correction value at a given phase position.
///
/// This function returns a small correction value that smooths a
/// discontinuity located at phase = 0.0 (i.e. a wrap-around point).
///
/// # Arguments
/// * `phase` — current oscillator phase in [0.0, 1.0)
/// * `dt`    — phase increment per sample (= frequency / sample_rate)
///
/// # How it works
///
/// Near phase = 0.0 (just AFTER the discontinuity), `t = phase / dt`
/// is a normalized distance in [0, 1) from the jump. We apply a
/// cubic polynomial that goes from +1 at t=0 down to 0 at t=1.
///
/// Near phase = 1.0 (just BEFORE the discontinuity), `t = (phase-1) / dt`
/// is a normalized distance in (-1, 0]. We apply a mirrored polynomial.
///
/// Outside those two windows, the correction is exactly 0.0 — no cost.
#[inline(always)]
fn poly_blep(phase: f32, dt: f32) -> f32 {
    if phase < dt {
        // Just AFTER the discontinuity
        // t goes from 0 (at the jump) to 1 (one sample later)
        let t = phase / dt;
        // Cubic: +1 at t=0, slope 0 at both ends → smooth onset
        2.0 * t - t * t - 1.0
    } else if phase > 1.0 - dt {
        // Just BEFORE the discontinuity
        // t goes from -1 (one sample before jump) to 0 (at the jump)
        let t = (phase - 1.0) / dt;
        // Mirror of the above
        t * t + 2.0 * t + 1.0
    } else {
        // Far from any discontinuity — no correction needed
        0.0
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Oscillator
// ─────────────────────────────────────────────────────────────────────────────

/// A band-limited oscillator supporting Sine, Sawtooth, Square, and Triangle.
///
/// All waveforms share the same phase accumulator. The waveform shape is
/// selected at construction time (or can be changed between notes).
///
/// Triangle is generated via leaky integration of the square wave:
///   triangle[n] = (1 - coeff) * square[n] + coeff * triangle[n-1]
/// This avoids a second PolyBLEP pass and gives a smooth triangle for free.
pub struct Oscillator {
    /// Current phase in [0.0, 1.0)
    phase: f32,
    /// Phase increment per sample = frequency / sample_rate
    phase_increment: f32,
    /// Selected waveform
    waveform: Waveform,
    /// Last triangle output — used for the integrator
    last_triangle: f32,
    /// Cached sample rate, needed when changing frequency
    sample_rate: u32,
}

impl Oscillator {
    pub fn new(frequency_hz: f32, sample_rate: u32, waveform: Waveform) -> Self {
        Self {
            phase: 0.0,
            phase_increment: frequency_hz / sample_rate as f32,
            waveform,
            last_triangle: 0.0,
            sample_rate,
        }
    }

    /// Change the frequency on the fly (e.g. for pitch slides or new notes).
    /// Safe to call between samples — no discontinuity introduced.
    pub fn set_frequency(&mut self, frequency_hz: f32) {
        self.phase_increment = frequency_hz / self.sample_rate as f32;
    }

    /// Change the waveform. Reset triangle integrator to avoid a DC pop.
    pub fn set_waveform(&mut self, waveform: Waveform) {
        self.waveform = waveform;
        self.last_triangle = 0.0;
    }

    /// Generate the next sample for the selected waveform.
    #[inline(always)]
    pub fn next_sample(&mut self) -> f32 {
        let sample = match self.waveform {
            Waveform::Sine     => self.next_sine(),
            Waveform::Sawtooth => self.next_saw(),
            Waveform::Square   => self.next_square(),
            Waveform::Triangle => self.next_triangle(),
        };

        // Advance and wrap phase
        self.phase += self.phase_increment;
        if self.phase >= 1.0 {
            self.phase -= 1.0;
        }

        sample
    }

    // ── Waveform generators ───────────────────────────────────────────────

    /// Sine: no discontinuity, no PolyBLEP needed.
    #[inline(always)]
    fn next_sine(&self) -> f32 {
        (self.phase * TAU).sin()
    }

    /// Sawtooth: ramps from -1 to +1 then jumps back to -1.
    ///
    /// Naive formula: `2 * phase - 1`
    /// One discontinuity at phase = 1.0 → 0.0, corrected by one PolyBLEP.
    #[inline(always)]
    fn next_saw(&self) -> f32 {
        let naive = 2.0 * self.phase - 1.0;
        naive - poly_blep(self.phase, self.phase_increment)
    }

    /// Square: +1 for first half-cycle, -1 for second half.
    ///
    /// Two discontinuities per cycle:
    ///   - At phase = 0.0: -1 → +1 (rising edge)
    ///   - At phase = 0.5: +1 → -1 (falling edge)
    ///
    /// For the falling edge we shift phase by 0.5 and wrap to [0,1)
    /// so we can reuse the same poly_blep() function.
    #[inline(always)]
    fn next_square(&self) -> f32 {
        let naive = if self.phase < 0.5 { 1.0 } else { -1.0 };

        // Correct the rising edge at phase = 0.0
        let correction_rise = poly_blep(self.phase, self.phase_increment);

        // Correct the falling edge at phase = 0.5
        // Shift phase by 0.5 and wrap into [0, 1) for poly_blep
        let phase_shifted = self.phase + 0.5;
        let phase_shifted = if phase_shifted >= 1.0 {
            phase_shifted - 1.0
        } else {
            phase_shifted
        };
        let correction_fall = poly_blep(phase_shifted, self.phase_increment);

        naive + correction_rise - correction_fall
    }

    /// Triangle: integrated & normalized square wave.
    ///
    /// Rather than computing triangle directly (which would need PolyBLEP
    /// on both edges), we run a leaky integrator on the PolyBLEP square.
    ///
    /// The integration formula:
    ///   tri[n] = dt * square[n] + (1 - dt) * tri[n-1]  ... times a scale
    ///
    /// We scale by `4 * phase_increment` so the output stays in [-1, +1].
    /// The "leaky" factor `(1 - phase_increment)` prevents DC drift.
    #[inline(always)]
    fn next_triangle(&mut self) -> f32 {
        let square = self.next_square();
        let dt = self.phase_increment;

        // Leaky integrator: accumulate the square, bleed off DC slowly
        let tri = dt * square + (1.0 - dt) * self.last_triangle;

        // Scale to restore amplitude to [-1, +1]
        // The factor 4.0 is derived from the integral of a unit square
        let output = tri * (4.0 * dt).recip().min(1.0);

        self.last_triangle = tri;
        output
    }
}
