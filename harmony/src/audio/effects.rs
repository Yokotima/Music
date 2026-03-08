/// A delay effect stores the input signal in a circular buffer and mixes
/// it back with the dry signal after a set time. Feedback routes the
/// delayed signal back into the delay input, creating repeating echoes
/// that decay naturally.
///
///   x[n] ──┬──────────────────────────────────────────► dry
///           │         ┌─────────────────────┐
///           └─► (+) ──► circular buffer[D] ──┬──► wet
///                ▲                           │
///                └───────── * feedback ──────┘
///
///
/// "A combination of all-pass filters and delay lines to simulate the
/// acoustic persistence of a space."
///
/// The classic Schroeder reverberator (1962) uses:
///   - 4 parallel comb filters (feedback delay lines) — simulate early reflections
///   - 2 series all-pass filters                     — diffuse the sound
///
/// Each comb filter i has a different delay length D_i and the same
/// feedback coefficient g. The all-pass filters smooth the echo density.
///
/// This is simpler than modern reverbs (Freeverb, convolution) but
/// entirely appropriate for an academic project — it produces a
/// recognizable, musical reverb tail.

// ─────────────────────────────────────────────────────────────────────────────
// Circular Buffer — shared by both effects
// ─────────────────────────────────────────────────────────────────────────────

/// A fixed-size circular (ring) buffer for audio delay lines.
///
/// Pre-allocated at construction — no heap activity in the audio callback.
struct CircularBuffer {
    data:       Vec<f32>,
    write_head: usize,
    len:        usize,
}

impl CircularBuffer {
    fn new(size_samples: usize) -> Self {
        Self {
            data:       vec![0.0; size_samples],
            write_head: 0,
            len:        size_samples,
        }
    }

    /// Write a sample at the current write head and advance it.
    #[inline(always)]
    fn write(&mut self, sample: f32) {
        self.data[self.write_head] = sample;
        self.write_head = (self.write_head + 1) % self.len;
    }

    /// Read a sample `delay` samples behind the write head.
    #[inline(always)]
    fn read(&self, delay: usize) -> f32 {
        let idx = (self.write_head + self.len - delay) % self.len;
        self.data[idx]
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Delay Effect
// ─────────────────────────────────────────────────────────────────────────────

/// Maximum delay time supported (2 seconds at 44100 Hz).
const MAX_DELAY_SAMPLES: usize = 44_100 * 2;

/// A stereo-compatible delay with feedback and wet/dry mix.
pub struct Delay {
    buffer:       CircularBuffer,
    delay_samples: usize,
    /// Feedback coefficient [0.0, 0.95]. Above 0.95 risks runaway.
    feedback:     f32,
    /// Wet/dry mix [0.0 = dry only, 1.0 = wet only]. 0.3 is a good default.
    wet_mix:      f32,
}

impl Delay {
    /// Create a delay effect.
    ///
    /// # Arguments
    /// * `delay_ms`  — delay time in milliseconds
    /// * `feedback`  — echo decay [0.0–0.95]
    /// * `wet_mix`   — wet signal level [0.0–1.0]
    /// * `sample_rate`
    pub fn new(delay_ms: f32, feedback: f32, wet_mix: f32, sample_rate: u32) -> Self {
        let delay_samples = ((delay_ms / 1000.0) * sample_rate as f32) as usize;
        let delay_samples = delay_samples.clamp(1, MAX_DELAY_SAMPLES);

        Self {
            buffer: CircularBuffer::new(MAX_DELAY_SAMPLES),
            delay_samples,
            feedback: feedback.clamp(0.0, 0.95),
            wet_mix:  wet_mix.clamp(0.0, 1.0),
        }
    }

    /// Update delay time without reallocating the buffer.
    pub fn set_delay_ms(&mut self, delay_ms: f32, sample_rate: u32) {
        self.delay_samples = ((delay_ms / 1000.0) * sample_rate as f32) as usize;
        self.delay_samples = self.delay_samples.clamp(1, MAX_DELAY_SAMPLES);
    }

    pub fn set_feedback(&mut self, feedback: f32) {
        self.feedback = feedback.clamp(0.0, 0.95);
    }

    pub fn set_wet_mix(&mut self, wet: f32) {
        self.wet_mix = wet.clamp(0.0, 1.0);
    }

    /// Process one sample.
    ///
    /// Reads the delayed sample, mixes it with feedback into the buffer,
    /// then returns the dry + wet mix.
    #[inline(always)]
    pub fn process(&mut self, input: f32) -> f32 {
        let delayed  = self.buffer.read(self.delay_samples);
        let feedback_sample = input + delayed * self.feedback;
        self.buffer.write(feedback_sample);

        let dry = input  * (1.0 - self.wet_mix);
        let wet = delayed * self.wet_mix;
        dry + wet
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Reverb — Schroeder Network
// ─────────────────────────────────────────────────────────────────────────────

/// Number of comb filters in the Schroeder reverberator.
const NUM_COMBS: usize = 4;
/// Number of all-pass filters in the Schroeder reverberator.
const NUM_ALLPASS: usize = 2;

/// Delay lengths for comb filters (in samples at 44100 Hz).
/// These prime-ish values are chosen to avoid resonance buildup
/// at harmonic multiples — a key insight from Schroeder's original paper.
const COMB_DELAYS: [usize; NUM_COMBS] = [1557, 1617, 1491, 1422];

/// Delay lengths for all-pass filters.
const ALLPASS_DELAYS: [usize; NUM_ALLPASS] = [225, 556];

/// Feedback coefficient for all-pass filters (standard Schroeder value).
const ALLPASS_FEEDBACK: f32 = 0.5;

/// A feedback comb filter: y[n] = x[n] + g * y[n - D]
struct CombFilter {
    buffer:   CircularBuffer,
    delay:    usize,
    feedback: f32,
}

impl CombFilter {
    fn new(delay_samples: usize, feedback: f32) -> Self {
        Self {
            buffer:   CircularBuffer::new(delay_samples + 1),
            delay:    delay_samples,
            feedback,
        }
    }

    #[inline(always)]
    fn process(&mut self, input: f32) -> f32 {
        let delayed = self.buffer.read(self.delay);
        let output  = input + self.feedback * delayed;
        self.buffer.write(output);
        output
    }
}

/// An all-pass filter: passes all frequencies but shifts their phase.
/// Used to diffuse the echo density after the comb filters.
///
/// Transfer function: H(z) = (-g + z^-D) / (1 - g * z^-D)
struct AllPassFilter {
    buffer:   CircularBuffer,
    delay:    usize,
    feedback: f32,
}

impl AllPassFilter {
    fn new(delay_samples: usize, feedback: f32) -> Self {
        Self {
            buffer:   CircularBuffer::new(delay_samples + 1),
            delay:    delay_samples,
            feedback,
        }
    }

    #[inline(always)]
    fn process(&mut self, input: f32) -> f32 {
        let delayed = self.buffer.read(self.delay);
        let output  = -self.feedback * input + delayed
                    + self.feedback * delayed;
        self.buffer.write(input + self.feedback * delayed);
        output
    }
}

/// Schroeder reverberator.
///
/// 4 parallel comb filters → summed → 2 series all-pass filters.
/// Wet/dry mix applied at output.
pub struct Reverb {
    combs:   [CombFilter;   NUM_COMBS],
    allpass: [AllPassFilter; NUM_ALLPASS],
    /// Room size controls the feedback of the comb filters [0.0–0.98].
    /// Higher = longer reverb tail.
    room_size: f32,
    /// Wet/dry mix [0.0–1.0].
    wet_mix:   f32,
}

impl Reverb {
    /// Create a reverb effect.
    ///
    /// # Arguments
    /// * `room_size` — reverb tail length [0.0–0.98]. 0.8 = large hall.
    /// * `wet_mix`   — wet signal level [0.0–1.0].
    pub fn new(room_size: f32, wet_mix: f32) -> Self {
        let room_size = room_size.clamp(0.0, 0.98);
        Self {
            combs: [
                CombFilter::new(COMB_DELAYS[0], room_size),
                CombFilter::new(COMB_DELAYS[1], room_size),
                CombFilter::new(COMB_DELAYS[2], room_size),
                CombFilter::new(COMB_DELAYS[3], room_size),
            ],
            allpass: [
                AllPassFilter::new(ALLPASS_DELAYS[0], ALLPASS_FEEDBACK),
                AllPassFilter::new(ALLPASS_DELAYS[1], ALLPASS_FEEDBACK),
            ],
            room_size,
            wet_mix: wet_mix.clamp(0.0, 1.0),
        }
    }

    pub fn set_room_size(&mut self, room_size: f32) {
        self.room_size = room_size.clamp(0.0, 0.98);
        for comb in self.combs.iter_mut() {
            comb.feedback = self.room_size;
        }
    }

    pub fn set_wet_mix(&mut self, wet: f32) {
        self.wet_mix = wet.clamp(0.0, 1.0);
    }

    /// Process one sample through the Schroeder network.
    #[inline(always)]
    pub fn process(&mut self, input: f32) -> f32 {
        // 4 parallel comb filters, summed
        let mut reverb_sum = 0.0_f32;
        for comb in self.combs.iter_mut() {
            reverb_sum += comb.process(input);
        }
        // Normalize by number of combs
        reverb_sum /= NUM_COMBS as f32;

        // 2 series all-pass filters for diffusion
        let mut diffused = reverb_sum;
        for ap in self.allpass.iter_mut() {
            diffused = ap.process(diffused);
        }

        // Wet/dry mix
        input * (1.0 - self.wet_mix) + diffused * self.wet_mix
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// EffectsChain — convenience wrapper used by the audio engine
// ─────────────────────────────────────────────────────────────────────────────

/// A serial effects chain: Delay → Reverb.
/// Applied to the summed output of the VoicePool.
pub struct EffectsChain {
    pub delay:  Delay,
    pub reverb: Reverb,
}

impl EffectsChain {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            // Delay: 375 ms, 40% feedback, 25% wet
            delay:  Delay::new(375.0, 0.4, 0.25, sample_rate),
            // Reverb: medium room, 30% wet
            reverb: Reverb::new(0.75, 0.30),
        }
    }

    /// Dry (no effects) chain for comparison.
    pub fn dry(sample_rate: u32) -> Self {
        Self {
            delay:  Delay::new(375.0, 0.0, 0.0, sample_rate),
            reverb: Reverb::new(0.0,  0.0),
        }
    }

    /// Process one sample through the full chain.
    #[inline(always)]
    pub fn process(&mut self, input: f32) -> f32 {
        let after_delay  = self.delay.process(input);
        let after_reverb = self.reverb.process(after_delay);
        after_reverb
    }
}
