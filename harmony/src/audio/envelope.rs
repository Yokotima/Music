#![allow(dead_code)]
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
/// We use the "one-pole filter" trick for smooth exponential curves:
///   output = target + (output - target) * coeff
///
/// where coeff = exp(-1 / (time_s * sample_rate))
/// The closer coeff is to 1.0, the slower the curve
///
/// Each Voice will own one Envelope. The voice feeds note-on / note-off
/// events in, and reads the amplitude multiplier out each sample.

//==========EnvelopeState==========
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EnvelopeState
{
    //Waiting for a note-on. Output = 0.0
    Idle,
    //Rising toward 1.0 (peak amplitude)
    Attack,
    //Falling from 1.0 toward the sustain level
    Decay,
    //Held at sustain level while key is pressed
    Sustain,
    //Fading to 0.0 after note-off
    Release,
}

//==========EnvelopeParams==========
/// All user-configurable parameters for one ADSR envelope.
/// These are shared across voices for the same instrument.
#[derive(Debug, Clone, Copy)]
pub struct EnvelopeParams
{
    pub attack_secs:f32,
    pub decay_secs:f32,
    pub sustain_level:f32,
    pub release_secs:f32,
}

//==========Envelope==========
pub struct Envelope
{
    state:EnvelopeState,
    level:f32,
    release_level:f32,
    attack_coeff:f32,
    decay_coeff:f32,
    release_coeff:f32,
    sample_rate:u32,
}

impl Envelope
{
    // Takes a sample rate. Returns a silent envelope in Idle state, ready to be triggered.
    pub fn new(sample_rate: u32) -> Self
    {
        Self
        {
            state:EnvelopeState::Idle,
            level:0.0,
            release_level:0.0,
            attack_coeff:0.0,
            decay_coeff:0.0,
            release_coeff:0.0,
            sample_rate,
        }
    }

    //==========Coefficient calculation==========
    // Takes a time in seconds. Returns the exponential curve coefficient for that duration.
    // Close to 1.0 = slow curve. Close to 0.0 = fast curve.
    fn make_coeff(time_secs: f32, sample_rate: u32) -> f32
    {
        let time_secs = time_secs.max(0.0001); //minimum 0.1 ms
        (-1.0 / (time_secs * sample_rate as f32)).exp()
    }

    // Takes ADSR params. Recomputes all three phase coefficients from the given timings.
    fn recompute_coeffs(&mut self, params: &EnvelopeParams)
    {
        self.attack_coeff = Self::make_coeff(params.attack_secs, self.sample_rate);
        self.decay_coeff = Self::make_coeff(params.decay_secs, self.sample_rate);
        self.release_coeff = Self::make_coeff(params.release_secs, self.sample_rate);
    }

    //==========Events==========
    // Takes ADSR params. Starts the Attack phase — the sound begins rising toward peak.
    pub fn note_on(&mut self, params: &EnvelopeParams)
    {
        self.recompute_coeffs(params);
        self.state = EnvelopeState::Attack;
    }

    // Takes ADSR params. Starts the Release phase — the sound begins fading to silence.
    pub fn note_off(&mut self, params: &EnvelopeParams)
    {
        if self.state != EnvelopeState::Idle
        {
            self.recompute_coeffs(params);
            self.release_level = self.level;
            self.state = EnvelopeState::Release;
        }
    }

    //==========Per-sample Processing==========
    //level = target + (level - target) * coeff
    // Takes ADSR params. Returns the current amplitude [0.0, 1.0] for this sample.
    // Multiply your oscillator output by this value to shape the volume over time.
    #[inline(always)]
    pub fn next_sample(&mut self, params: &EnvelopeParams) -> f32
    {
        match self.state
        {
            EnvelopeState::Idle =>
            {
                self.level = 0.0;
            }

            EnvelopeState::Attack =>
            {
                self.level = 1.0 + (self.level - 1.0) * self.attack_coeff;
                if self.level >= 0.999
                {
                    self.level = 1.0;
                    self.state = EnvelopeState::Decay;
                }
            }

            EnvelopeState::Decay =>
            {
                self.level = params.sustain_level + (self.level - params.sustain_level) * self.decay_coeff;
                if (self.level - params.sustain_level).abs() < 0.001
                {
                    self.level = params.sustain_level;
                    self.state = EnvelopeState::Sustain;
                }
            }

            EnvelopeState::Sustain =>
            {
                self.level = params.sustain_level;
            }

            EnvelopeState::Release =>
            {
                self.level = 0.0 + (self.level -0.0) * self.release_coeff;
                if self.level < 0.0001
                {
                    self.level = 0.0;
                    self.state = EnvelopeState::Idle;
                }
            }
        }

        self.level
    }

    //==========State queries==========
    // Takes nothing. Returns true when the envelope is silent — voice can be recycled.
    pub fn is_idle(&self) -> bool
    {
        self.state == EnvelopeState::Idle
    }

    // Takes nothing. Returns the current amplitude level [0.0, 1.0].
    pub fn level(&self) -> f32
    {
        self.level
    }

    // Takes nothing. Returns the current phase (Idle/Attack/Decay/Sustain/Release).
    pub fn state(&self) -> EnvelopeState
    {
        self.state
    }
}
