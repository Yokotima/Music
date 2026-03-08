/// audio/envelope.rs
///
/// Step 3 — ADSR Envelope Generator
///
/// ## What is an ADSR envelope?
///
/// A note is not just a constant amplitude — it evolves over time.
/// The ADSR model divides this evolution into 4 phases:
///
///   Attack  — how fast the sound rises to peak amplitude after note-on
///   Decay   — how fast it falls from peak to the sustain level
///   Sustain — the amplitude held as long as the key is pressed (0.0–1.0)
///   Release — how fast it fades to silence after note-off
///
/// ## Why exponential curves?
///
/// Linear ramps sound unnatural because human hearing is logarithmic.
/// A sound that drops linearly from 1.0 to 0.5 sounds like it barely
/// changed. Exponential curves match our perception: equal time → equal
/// perceived loudness change.
///
/// We use the "one-pole filter" trick for smooth exponential curves:
///   output = target + (output - target) * coeff
///
/// where `coeff = exp(-1 / (time_s * sample_rate))`.
/// The closer coeff is to 1.0, the slower the curve.
///
/// ## Integration with the voice pool (Step 4)
///
/// Each Voice will own one Envelope. The voice feeds note-on / note-off
/// events in, and reads the amplitude multiplier out each sample.

// ─────────────────────────────────────────────────────────────────────────────
// EnvelopeState
// ─────────────────────────────────────────────────────────────────────────────

/// The current phase of the envelope state machine.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnvelopeState {
    /// Waiting for a note-on. Output = 0.0.
    Idle,
    /// Rising toward 1.0 (peak amplitude).
    Attack,
    /// Falling from 1.0 toward the sustain level.
    Decay,
    /// Held at sustain level while key is pressed.
    Sustain,
    /// Fading to 0.0 after note-off.
    Release,
}

// ─────────────────────────────────────────────────────────────────────────────
// EnvelopeParams
// ─────────────────────────────────────────────────────────────────────────────

/// All user-configurable parameters for one ADSR envelope.
/// These are shared across voices for the same instrument.
#[derive(Debug, Clone, Copy)]
pub struct EnvelopeParams {
    /// Attack time in seconds. Typical: 0.001 (piano) – 0.5 (pad)
    pub attack_secs: f32,
    /// Decay time in seconds. Typical: 0.05 – 0.3
    pub decay_secs: f32,
    /// Sustain level in [0.0, 1.0]. 1.0 = no decay at all.
    pub sustain_level: f32,
    /// Release time in seconds. Typical: 0.05 (staccato) – 2.0 (pad)
    pub release_secs: f32,
}

impl EnvelopeParams {
    /// A generic default: punchy attack, medium decay, full sustain, short release.
    pub fn default_synth() -> Self {
        Self {
            attack_secs:   0.01,
            decay_secs:    0.1,
            sustain_level: 0.8,
            release_secs:  0.2,
        }
    }

    /// Piano-like: near-instant attack, long natural decay, no sustain.
    pub fn piano() -> Self {
        Self {
            attack_secs:   0.002,
            decay_secs:    1.5,
            sustain_level: 0.0,
            release_secs:  0.3,
        }
    }

    /// Pad: slow fade in, held sustain, long release.
    pub fn pad() -> Self {
        Self {
            attack_secs:   0.4,
            decay_secs:    0.2,
            sustain_level: 0.9,
            release_secs:  1.5,
        }
    }

    /// Pluck: instant attack, fast decay, no sustain.
    pub fn pluck() -> Self {
        Self {
            attack_secs:   0.001,
            decay_secs:    0.08,
            sustain_level: 0.0,
            release_secs:  0.05,
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Envelope
// ─────────────────────────────────────────────────────────────────────────────

/// Per-voice ADSR envelope generator.
///
/// Usage:
/// ```
/// let mut env = Envelope::new(44100);
/// env.note_on(&params);
/// // each sample:
/// let amp = env.next_sample(&params);
/// // on key release:
/// env.note_off(&params);
/// // when env.is_idle() == true, the voice can be recycled
/// ```
pub struct Envelope {
    /// Current state machine phase
    state: EnvelopeState,
    /// Current output amplitude [0.0, 1.0]
    level: f32,
    /// The amplitude at which release started (to avoid a pop when releasing)
    release_level: f32,
    /// Pre-computed coefficients (recomputed on note_on if params change)
    attack_coeff:  f32,
    decay_coeff:   f32,
    release_coeff: f32,
    /// Cached sample rate
    sample_rate: u32,
}

impl Envelope {
    pub fn new(sample_rate: u32) -> Self {
        Self {
            state:         EnvelopeState::Idle,
            level:         0.0,
            release_level: 0.0,
            attack_coeff:  0.0,
            decay_coeff:   0.0,
            release_coeff: 0.0,
            sample_rate,
        }
    }

    // ── Coefficient calculation ───────────────────────────────────────────

    /// Compute the one-pole filter coefficient for a given time.
    ///
    /// `coeff = exp(-1 / (time_s * sample_rate))`
    ///
    /// This gives a curve that reaches ~63% of its target in `time_s` seconds,
    /// which matches the RC circuit model used in analog synthesizers.
    ///
    /// We clamp the minimum time to avoid division by zero.
    fn make_coeff(time_secs: f32, sample_rate: u32) -> f32 {
        let time_secs = time_secs.max(0.0001); // minimum 0.1 ms
        (-1.0 / (time_secs * sample_rate as f32)).exp()
    }

    fn recompute_coeffs(&mut self, params: &EnvelopeParams) {
        self.attack_coeff  = Self::make_coeff(params.attack_secs,  self.sample_rate);
        self.decay_coeff   = Self::make_coeff(params.decay_secs,   self.sample_rate);
        self.release_coeff = Self::make_coeff(params.release_secs, self.sample_rate);
    }

    // ── Events ────────────────────────────────────────────────────────────

    /// Trigger a note-on. Transitions to Attack from any state.
    ///
    /// We do NOT reset level to 0 — if a note is re-triggered while still
    /// in release, we start the attack from the current level to avoid a
    /// sudden jump (this is called "legato" re-trigger behaviour).
    pub fn note_on(&mut self, params: &EnvelopeParams) {
        self.recompute_coeffs(params);
        self.state = EnvelopeState::Attack;
    }

    /// Trigger a note-off. Transitions to Release, capturing current level.
    pub fn note_off(&mut self, params: &EnvelopeParams) {
        if self.state != EnvelopeState::Idle {
            self.recompute_coeffs(params);
            self.release_level = self.level;
            self.state = EnvelopeState::Release;
        }
    }

    // ── Per-sample processing ─────────────────────────────────────────────

    /// Advance the envelope by one sample and return the current amplitude.
    ///
    /// The one-pole formula used in each active stage:
    ///   level = target + (level - target) * coeff
    ///
    /// This asymptotically approaches `target`. We detect arrival by checking
    /// if we are "close enough" to the target (within a small epsilon).
    #[inline(always)]
    pub fn next_sample(&mut self, params: &EnvelopeParams) -> f32 {
        match self.state {
            EnvelopeState::Idle => {
                self.level = 0.0;
            }

            EnvelopeState::Attack => {
                // Approach 1.0 (slightly above to ensure we reach it)
                self.level = 1.0 + (self.level - 1.0) * self.attack_coeff;
                // Transition to Decay once we are close enough to peak
                if self.level >= 0.999 {
                    self.level = 1.0;
                    self.state = EnvelopeState::Decay;
                }
            }

            EnvelopeState::Decay => {
                // Approach the sustain level
                self.level = params.sustain_level
                    + (self.level - params.sustain_level) * self.decay_coeff;
                // Transition to Sustain once close enough
                if (self.level - params.sustain_level).abs() < 0.001 {
                    self.level = params.sustain_level;
                    self.state = EnvelopeState::Sustain;
                }
            }

            EnvelopeState::Sustain => {
                // Hold steady — note_off() will move us to Release
                self.level = params.sustain_level;
            }

            EnvelopeState::Release => {
                // Approach 0.0 from release_level
                self.level = self.level * self.release_coeff;
                // Transition to Idle once silent enough
                if self.level < 0.0001 {
                    self.level = 0.0;
                    self.state = EnvelopeState::Idle;
                }
            }
        }

        self.level
    }

    // ── State queries ─────────────────────────────────────────────────────

    /// Returns true when the envelope is silent and the voice can be recycled.
    pub fn is_idle(&self) -> bool {
        self.state == EnvelopeState::Idle
    }

    /// Current output level (same as last next_sample() return value).
    pub fn level(&self) -> f32 {
        self.level
    }

    /// Current state (useful for debugging / UI display).
    pub fn state(&self) -> EnvelopeState {
        self.state
    }
}
