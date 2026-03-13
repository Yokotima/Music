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
}

impl Oscillator
{
    // Takes frequency, sample rate, waveform
    pub fn new(frequency_hz: f32, sample_rate: u32, waveform: Waveform) -> Self
    {
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
        }
    }

    // Update pitch
    pub fn set_frequency(&mut self, frequency_hz: f32)
    {
        self.phase_increment = frequency_hz / self.sample_rate as f32;
    }

    // Change waveform
    pub fn set_waveform(&mut self, waveform: Waveform)
    {
        self.waveform = waveform;
        self.last_triangle = 0.0;
    }

    //==========Main sample generation==========
    #[inline(always)]
    pub fn next_sample(&mut self) -> f32
    {
        let dt = self.phase_increment;

        let s1 = self.sample_from_phase(self.phase1);
        let s2 = self.sample_from_phase(self.phase2);
        let s3 = self.sample_from_phase(self.phase3);

        let sample = (s1 + s2 + s3) * 0.333333333333;

        // advance phases
        self.phase1 += dt;
        self.phase2 += dt * (1.0 + self.detune);
        self.phase3 += dt * (1.0 - self.detune);

        if self.phase1 >= 1.0 { self.phase1 -= 1.0; }
        if self.phase2 >= 1.0 { self.phase2 -= 1.0; }
        if self.phase3 >= 1.0 { self.phase3 -= 1.0; }

        sample
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
        }
    }
}
