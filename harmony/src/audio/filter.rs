/// ## What is a biquad filter?
///
/// A biquad (bi-quadratic) filter is the fundamental building block of
/// digital audio processing. It implements a second-order IIR filter
/// (Infinite Impulse Response) via the difference equation:
///
///   y[n] = b0*x[n] + b1*x[n-1] + b2*x[n-2]
///          - a1*y[n-1] - a2*y[n-2]
///
/// where x is input, y is output, and {b0,b1,b2,a1,a2} are coefficients.
///
/// - Computationally cheap: 5 multiplications + 4 additions per sample
/// - Numerically stable for audio frequency ranges
/// - Can implement LP, HP, BP, notch, peak, shelf by changing coefficients
/// - Coefficients derived via the bilinear transform (s-domain → z-domain)
///
/// ## Coefficient design
///
/// We use the Audio EQ Cookbook formulas by Robert Bristow-Johnson,
/// the standard reference for biquad audio filter design.
/// Each filter type maps cutoff frequency + Q factor to coefficients.
///
/// "Coefficients are recalculated dynamically when parameters change,
/// with smooth interpolation to avoid audible clicks."
///
/// We interpolate coefficients over 64 samples when they change,
/// preventing the discontinuity that causes clicks.

//==========FilterType==========
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FilterType
{
    LowPass,
    HighPass,
    BandPass,
    Notch,
    Bypass,
}

//==========BiquadCoeffs — the 5 filter coefficients==========
#[derive(Debug, Clone, Copy)]
struct BiquadCoeffs
{
    b0:f32, 
    b1:f32, 
    b2:f32,
    a1:f32, 
    a2:f32,
}

impl BiquadCoeffs
{
    // Takes nothing. Returns coefficients that pass the signal through unchanged.
    fn bypass() -> Self
    {
        Self 
        { 
            b0:1.0, 
            b1:0.0, 
            b2:0.0, 
            a1:0.0, 
            a2:0.0
        }
    }

    /// Compute coefficients for a given filter type.
    ///
    /// Based on the Audio EQ Cookbook by Robert Bristow-Johnson.
    /// https://www.w3.org/TR/audio-eq-cookbook/
    ///
    /// # Arguments
    /// * `filter_type`  — which filter shape to compute
    /// * `cutoff_hz`    — cutoff / center frequency in Hz
    /// * `q`            — quality factor (resonance). 0.707 = Butterworth (flat)
    /// * `sample_rate`  — audio sample rate in Hz
    // Takes filter type, cutoff frequency and Q. Returns the 5 biquad coefficients for that shape.
    fn compute(filter_type: FilterType, cutoff_hz: f32, q: f32, sample_rate: u32) -> Self
    {
        if filter_type == FilterType::Bypass
        {
            return Self::bypass();
        }

        // Clamp inputs to safe ranges
        let fs = sample_rate as f32;
        let f0 = cutoff_hz.clamp(20.0, fs * 0.499); // never exceed Nyquist
        let q = q.max(0.01);                        // avoid division by zero

        // Intermediate variables (Audio EQ Cookbook notation)
        let w0 = 2.0 * std::f32::consts::PI * f0 / fs;
        let cos_w = w0.cos();
        let sin_w = w0.sin();
        let alpha = sin_w / (2.0 * q);

        // Compute raw coefficients per filter type
        let (b0, b1, b2, a0, a1, a2) = match filter_type
        {
            FilterType::LowPass =>
            {
                let b0 = (1.0 - cos_w) / 2.0;
                let b1 = 1.0 - cos_w;
                let b2 = (1.0 - cos_w) / 2.0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_w;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            FilterType::HighPass =>
            {
                let b0 = (1.0 + cos_w) / 2.0;
                let b1 = -(1.0 + cos_w);
                let b2 = (1.0 + cos_w) / 2.0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_w;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            FilterType::BandPass =>
            {
                // BPF (constant skirt gain, peak gain = Q)
                let b0 =  sin_w / 2.0;
                let b1 =  0.0;
                let b2 = -sin_w / 2.0;
                let a0 =  1.0 + alpha;
                let a1 = -2.0 * cos_w;
                let a2 =  1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            FilterType::Notch =>
            {
                let b0 = 1.0;
                let b1 = -2.0 * cos_w;
                let b2 = 1.0;
                let a0 = 1.0 + alpha;
                let a1 = -2.0 * cos_w;
                let a2 = 1.0 - alpha;
                (b0, b1, b2, a0, a1, a2)
            }
            FilterType::Bypass => unreachable!(),
        };

        // Normalize by a0 (standard biquad normalization)
        Self
        {
            b0:b0 / a0,
            b1:b1 / a0,
            b2:b2 / a0,
            a1:a1 / a0,
            a2:a2 / a0,
        }
    }

    /// Linear interpolation between two coefficient sets.
    /// Used for smooth parameter transitions (avoids clicks).
    // Takes another coefficient set and a blend factor t [0.0-1.0]. Returns the interpolated set.
    fn lerp(self, other: Self, t: f32) -> Self
    {
        Self
        {
            b0:self.b0 + (other.b0 - self.b0) * t,
            b1:self.b1 + (other.b1 - self.b1) * t,
            b2:self.b2 + (other.b2 - self.b2) * t,
            a1:self.a1 + (other.a1 - self.a1) * t,
            a2:self.a2 + (other.a2 - self.a2) * t,
        }
    }
}

//==========BiquadFilter==========
/// Number of samples over which to interpolate coefficient changes.
/// 64 samples ≈ 1.5 ms at 44100 Hz — inaudible transition, no clicks.
const SMOOTH_SAMPLES: u32 = 64;

/// A single biquad filter with smooth parameter changes.
///
/// Each Voice will own one BiquadFilter in Step 7.
/// The filter operates on one sample at a time (no block processing)
/// to stay compatible with the per-sample voice loop.
pub struct BiquadFilter
{
    current:BiquadCoeffs,
    target:BiquadCoeffs,
    x1:f32,
    x2:f32,
    y1:f32,
    y2:f32,
    smooth_remaining:u32,
    filter_type:FilterType,
    cutoff_hz:f32,
    q:f32,
    sample_rate:u32,
}

impl BiquadFilter
{
    // Takes a sample rate. Returns a new filter in bypass mode (signal passes through unchanged).
    pub fn new(sample_rate: u32) -> Self
    {
        let bypass = BiquadCoeffs::bypass();
        Self
        {
            current:bypass,
            target:bypass,
            x1:0.0,
            x2:0.0,
            y1:0.0,
            y2:0.0,
            smooth_remaining:0,
            filter_type:FilterType::Bypass,
            cutoff_hz:1000.0,
            q:0.707,
            sample_rate,
        }
    }

    /// Update filter parameters. Triggers smooth coefficient interpolation.
    ///
    /// Safe to call from the audio callback — no allocation, no branching
    /// other than a few comparisons.
    // Takes filter type, cutoff Hz and Q. Updates the filter smoothly over 64 samples (no click).
    pub fn set_params(&mut self, filter_type: FilterType, cutoff_hz: f32, q: f32)
    {
        // Skip recomputation if nothing changed
        if filter_type == self.filter_type && (cutoff_hz - self.cutoff_hz).abs() < 0.01 && (q - self.q).abs() < 0.001
        {
            return;
        }

        self.filter_type = filter_type;
        self.cutoff_hz = cutoff_hz;
        self.q = q;

        // Compute new target coefficients
        self.target = BiquadCoeffs::compute(filter_type, cutoff_hz, q, self.sample_rate);
        // Start smooth interpolation
        self.smooth_remaining = SMOOTH_SAMPLES;
    }

    /// Process one input sample and return the filtered output.
    ///
    /// Implements the Direct Form I difference equation:
    ///   y[n] = b0*x[n] + b1*x[n-1] + b2*x[n-2]
    ///          - a1*y[n-1] - a2*y[n-2]
    ///
    /// Direct Form I is preferred over Form II here because it has
    /// better numerical behaviour when coefficients change mid-stream.
    // Takes one raw audio sample. Returns that sample with the filter applied.
    #[inline(always)]
    pub fn process(&mut self, x: f32) -> f32
    {
        // Advance coefficient interpolation if active
        if self.smooth_remaining > 0
        {
            let t = 1.0 - (self.smooth_remaining as f32 / SMOOTH_SAMPLES as f32);
            self.current = self.current.lerp(self.target, t);
            self.smooth_remaining -= 1;
            // Snap to target on last step to avoid float drift
            if self.smooth_remaining == 0
            {
                self.current = self.target;
            }
        }

        let c = &self.current;
        // Difference equation
        let y = c.b0 * x + c.b1 * self.x1 + c.b2 * self.x2 - c.a1 * self.y1 - c.a2 * self.y2;

        // Shift delay lines
        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }

    /// Reset the filter state (delay lines).
    /// Call this when a voice is re-triggered to avoid residual noise.
    // Takes nothing. Clears the internal memory — call on note_on to avoid leftover noise.
    pub fn reset(&mut self)
    {
        self.x1 = 0.0;
        self.x2 = 0.0;
        self.y1 = 0.0;
        self.y2 = 0.0;
    }
}
//==========FilterPreset — ready-to-use configs for each instrument==========
#[derive(Debug, Clone, Copy)]
pub struct FilterPreset
{
    pub filter_type:FilterType,
    pub cutoff_hz:f32,
    pub q:f32,
}

impl FilterPreset
{
    // Takes nothing. Returns a bypass preset — the signal is not filtered at all.
    pub fn bypass() -> Self
    {
        Self 
        {
            filter_type:FilterType::Bypass,
            cutoff_hz:20_000.0,
            q:0.707
        }
    }

    // Takes nothing. Returns a gentle low-pass at 4kHz for a natural piano tone.
    pub fn piano() -> Self
    {
        Self
        { 
            filter_type:FilterType::LowPass,
            cutoff_hz:4_000.0,
            q:0.707
        }
    }

    // Takes nothing. Returns a soft low-pass at 2.5kHz for a breathy flute tone.
    pub fn flute() -> Self
    {
        Self 
        { 
            filter_type:FilterType::LowPass,
            cutoff_hz:2_500.0, 
            q:0.6
        }
    }

    // Takes nothing. Returns a heavy low-pass at 800Hz — keeps only the deep bass frequencies.
    pub fn bass() -> Self
    {
        Self
        {
            filter_type:FilterType::LowPass,
            cutoff_hz:800.0,
            q:1.2
        }
    }

    // Takes nothing. Returns a warm low-pass at 1.2kHz with light resonance for pad texture.
    pub fn pad() -> Self
    {
        Self
        { 
            filter_type:FilterType::LowPass,
            cutoff_hz:1_200.0, 
            q:1.5
        }
    }

    // Takes nothing. Returns a resonant low-pass at 3kHz — gives the synth lead its bite.
    pub fn lead() -> Self
    {
        Self
        { 
            filter_type:FilterType::LowPass, 
            cutoff_hz:3_000.0, 
            q:3.0
        }
    }
}
