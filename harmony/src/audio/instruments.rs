/// Each instrument is a complete configuration:
///   - Waveform      (oscillator shape)
///   - EnvelopeParams (ADSR)
///   - FilterPreset  (tonal character)
///   - EffectConfig  (delay + reverb amounts)
///
///   1. Piano
///   2. Flute
///   3. Bass
///   4. Pad
///   5. Lead
///
/// ## Architecture
///
/// An `Instrument` struct holds all parameters. 
/// The `InstrumentEngine` owns a `VoicePool` + `EffectsChain` and reconfigures they changes
/// instruments.

use super::effects::EffectsChain;
use super::envelope::EnvelopeParams;
use super::filter::FilterPreset;
use super::oscillator::Waveform;
use super::voice::VoicePool;

//==========Instrument Kind==========
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InstrumentKind
{
    Piano,
    Flute,
    Bass,
    Pad,
    Lead,
}

impl InstrumentKind
{
    // Takes nothing. Returns the instrument name as a string for display or logging.
    pub fn name(&self) -> &'static str
    {
        match self
        {
            Self::Piano => "Piano",
            Self::Flute => "Flute",
            Self::Bass => "Bass",
            Self::Pad => "Pad",
            Self::Lead => "Lead",
        }
    }

    //All instruments in order for test or demo
    // Takes nothing. Returns all five instrument kinds in order — useful for cycling demos.
    pub fn all() -> [InstrumentKind; 5]
    {
        [Self::Piano, Self::Flute, Self::Bass, Self::Pad, Self::Lead]
    }
}

//==========Effect Configue==========
#[derive(Debug, Clone, Copy)]
pub struct EffectConfig
{
    pub delay_ms:f32,
    pub feedback:f32,
    pub delay_wet:f32,
    pub room_size:f32,
    pub reverb_wet:f32,
}

impl EffectConfig
{
    // Takes nothing. Returns a dry config — no delay, no reverb. Used for Piano and Bass.
    pub fn none() -> Self
    {
        Self
        { 
            delay_ms:250.0,
            feedback:0.0,
            delay_wet:0.0,
            room_size:0.5,
            reverb_wet:0.0,
        }
    }
}

//==========Instrument==========
#[derive(Debug, Clone, Copy)]
pub struct Instrument
{
    pub kind:InstrumentKind,
    pub waveform:Waveform,
    pub envelope:EnvelopeParams,
    pub filter:FilterPreset,
    pub effects:EffectConfig,
}

impl Instrument 
{
    //==========Piano========== 
    // Takes nothing. Returns the Piano instrument bundle: Triangle wave, fast attack, long decay, no effects.
    pub fn piano() -> Self 
    {
        Self 
        {
            kind:InstrumentKind::Piano,
            waveform:Waveform::Triangle, //Triangle
            envelope:EnvelopeParams
            {
                attack_secs:0.002, //0.002
                decay_secs:1.8, // 1.8
                sustain_level:0.0,
                release_secs:0.3,
            },
            filter:FilterPreset::piano(),   // LP 4kHz Q=0.707
            //effects:EffectConfig::none(),
            effects:EffectConfig
            {
            delay_ms:0.0,
            feedback:0.0,
            delay_wet:0.0,
            room_size:0.45,
            reverb_wet:0.08,
            },
        }
    }
    //==========Flute==========
    // Takes nothing. Returns the Flute instrument bundle: Sine wave, slow attack, light reverb.
    pub fn flute() -> Self 
    {
        Self
        {
            kind:InstrumentKind::Flute,
            waveform:Waveform::Sine,
            envelope:EnvelopeParams
            {
                attack_secs:0.08,
                decay_secs:0.1,
                sustain_level:0.85,
                release_secs:0.25,
            },
            filter:FilterPreset::flute(),   // LP 2.5kHz Q=0.6
            effects:EffectConfig
            {
                delay_ms:0.0,
                feedback:0.0,
                delay_wet:0.0,
                room_size:0.6,
                reverb_wet:0.20,
            },
        }
    }
    //==========Bass==========
    // Takes nothing. Returns the Bass instrument bundle: Sawtooth wave, heavy LP filter, no effects.
    pub fn bass() -> Self
    {
        Self
        {
            kind:InstrumentKind::Bass,
            waveform:Waveform::Sawtooth,
            envelope:EnvelopeParams{
                attack_secs:0.003,
                decay_secs:0.08,
                sustain_level:0.7,
                release_secs:0.08,
            },
            filter:FilterPreset::bass(),    // LP 800Hz Q=1.2
            effects:EffectConfig::none(),
        }
    }

    //==========Pad==========
    // Takes nothing. Returns the Pad instrument bundle: Sawtooth wave, slow attack, heavy reverb.
    pub fn pad() -> Self
    {
        Self
        {
            kind:InstrumentKind::Pad,
            waveform:Waveform::Sawtooth,
            envelope:EnvelopeParams
            {
                attack_secs:0.2,   // reduced from 0.4 — faster response, less CPU
                decay_secs:0.2,
                sustain_level:0.80,
                release_secs:0.8,   // reduced from 1.8 — voices freed sooner
            },
            filter:FilterPreset::pad(),
            effects:EffectConfig
            {
                delay_ms:375.0,
                feedback:0.30,
                delay_wet:0.15,
                room_size:0.65,     // reduced from 0.85 — less reverb tail = less CPU
                reverb_wet:0.25,     // reduced from 0.40
            },
        }
    }
    //==========Lead==========
    // Takes nothing. Returns the Lead instrument bundle: Square wave, resonant filter, medium delay.
    pub fn lead() -> Self
    {
        Self
        {
            kind:InstrumentKind::Lead,
            waveform:Waveform::Square,
            envelope:EnvelopeParams
            {
                attack_secs:0.005,
                decay_secs:0.15,
                sustain_level:0.75,
                release_secs:0.15,
            },
            filter:FilterPreset::lead(),    // LP 3kHz Q=3.0
            effects:EffectConfig
            {
                delay_ms:300.0,
                feedback:0.45,
                delay_wet:0.25,
                room_size:0.5,
                reverb_wet:0.15,
            },
        }
    }

    // Takes an InstrumentKind. Returns the matching Instrument bundle.
    pub fn get(kind: InstrumentKind) -> Self
    {
        match kind
        {
            InstrumentKind::Piano => Self::piano(),
            InstrumentKind::Flute => Self::flute(),
            InstrumentKind::Bass  => Self::bass(),
            InstrumentKind::Pad   => Self::pad(),
            InstrumentKind::Lead  => Self::lead(),
        }
    }
}

//==========All sort of engine for Instrument==========
pub struct InstrumentEngine
{
    pub pool:VoicePool,
    pub fx:EffectsChain,
    pub current:InstrumentKind,
    sample_rate:u32,
}

impl InstrumentEngine
{
    // Takes an instrument kind and sample rate. Returns a fully configured engine ready to play.
    pub fn new(kind: InstrumentKind, sample_rate: u32) -> Self
    {
        let inst = Instrument::get(kind);
        let mut pool = VoicePool::new(sample_rate, inst.waveform, inst.envelope);

        pool.set_filter(inst.filter);
        let fx = Self::build_fx(&inst.effects, sample_rate);
        println!("[Instrument] Loaded: {}", kind.name());

        Self
        { 
            pool,
            fx,
            current:kind,
            sample_rate
        }
    }

    // Takes an instrument kind. Rebuilds the voice pool and effects chain for the new instrument.
    pub fn set_instrument(&mut self, kind: InstrumentKind)
    {
        let inst = Instrument::get(kind);
        self.current = kind;
        self.pool = VoicePool::new(self.sample_rate, inst.waveform, inst.envelope);
        self.pool.set_filter(inst.filter);
        self.fx = Self::build_fx(&inst.effects, self.sample_rate);
        println!("[instrument] Switched to: {}", kind.name());
    }

    // Takes nothing. Returns one audio sample: voices summed, then delay and reverb applied.
    // This is THE function that produces all sound — call it 44100 times per second.
    #[inline(always)]
    pub fn next_sample(&mut self) -> f32
    {
        let dry = self.pool.next_sample();
        self.fx.process(dry)
    }

    // Takes an EffectConfig and sample rate. Returns a configured EffectsChain for that instrument.
    fn build_fx(cfg: &EffectConfig, sample_rate: u32) -> EffectsChain
    {
        use super::effects::{Delay, Reverb};
        EffectsChain
        {
            delay:Delay::new(cfg.delay_ms, cfg.feedback, cfg.delay_wet, sample_rate),
            reverb:Reverb::new(cfg.room_size, cfg.reverb_wet),
        }
    }
}
