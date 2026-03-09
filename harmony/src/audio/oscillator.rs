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
pub struct Oscillator
{
    phase:f32,
    phase_increment:f32,
    waveform:Waveform,
    last_triangle:f32,
    sample_rate:u32,
}

impl Oscillator
{
    pub fn new(frequency_hz: f32, sample_rate: u32, waveform: Waveform) -> Self
    {
        Self
        {
            phase:0.0,
            phase_increment:frequency_hz / (sample_rate as f32),
            waveform,
            last_triangle:0.0,
            sample_rate,
        }
    }

    pub fn set_frequency(&mut self, frequency_hz: f32)
    {
        self.phase_increment = frequency_hz / (self.sample_rate) as f32;
    }

    pub fn set_waveform(&mut self, waveform: Waveform)
    {
        self.waveform = waveform;
        self.last_triangle = 0.0;
    }

    #[inline(always)]
    pub fn next_sample(&mut self) -> f32
    {
        let sample = match self.waveform
        {
            Waveform::Sine => self.next_sine(),
            Waveform::Sawtooth => self.next_saw(),
            Waveform::Square => self.next_square(),
            Waveform::Triangle => self.next_triangle(),
        };

        self.phase += self.phase_increment;
        if self.phase >= 1.0
        {
            self.phase -= 1.0;
        }
        sample
    }

    //==========Waveform generators==========

    #[inline(always)]
    fn next_sine(&self) -> f32
    {
        (self.phase * TAU).sin()
    }

    #[inline(always)]
    fn next_saw(&self) -> f32
    {
        let naive = 2.0 * self.phase - 1.0;
        naive - poly_blep(self.phase, self.phase_increment)
    }

    #[inline(always)]
    fn next_square(&self) -> f32
    {
        let naive = if self.phase < 0.5 { 1.0 } else { -1.0 };

        let correction_rise = poly_blep(self.phase, self.phase_increment);

        let phase_shifted = self.phase + 0.5;
        let phase_shifted = 
            if phase_shifted >= 1.0
            {
                phase_shifted - 1.0
            } 
            else
            {
                phase_shifted
            };
        let correction_fall = poly_blep(phase_shifted, self.phase_increment);
        naive + correction_rise - correction_fall
    }

    #[inline(always)]
    fn next_triangle(&mut self) -> f32
    {
        let square = self.next_square();
        let dt = self.phase_increment;

        let tri = dt * square + (1.0 - dt) * self.last_triangle;

        let output = tri * (4.0 * dt).recip().min(1.0);

        self.last_triangle = tri;
        output
    }
}
