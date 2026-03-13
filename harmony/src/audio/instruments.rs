/// audio/instruments.rs — Structs partagés et moteur audio.
///
/// Ce fichier ne contient AUCUNE valeur de paramètre sonore.
/// Chaque instrument a son propre fichier de config :
///   piano.rs | flute.rs | bass.rs | pad.rs | lead.rs
///
/// Architecture :
///   InstrumentKind  — enum des 5 instruments
///   EffectConfig    — struct delay + reverb (utilisé dans chaque config)
///   Instrument      — bundle complet (waveform + ADSR + filtre + effets)
///   InstrumentEngine — moteur audio : VoicePool + EffectsChain

use super::effects::EffectsChain;
use super::envelope::EnvelopeParams;
use super::filter::FilterPreset;
use super::oscillator::Waveform;
use super::voice::VoicePool;

//==========InstrumentKind==========

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
            Self::Bass  => "Bass",
            Self::Pad   => "Pad",
            Self::Lead  => "Lead",
        }
    }

    // Takes nothing. Returns all five instrument kinds in order.
    pub fn all() -> [InstrumentKind; 5]
    {
        [Self::Piano, Self::Flute, Self::Bass, Self::Pad, Self::Lead]
    }
}

//==========EffectConfig==========
// Delay + Reverb amounts. All fields exposed — change them in each instrument file.

#[derive(Debug, Clone, Copy)]
pub struct EffectConfig
{
    pub delay_ms:   f32,  // delay time in milliseconds
    pub feedback:   f32,  // echo repetition amount  [0.0 – 0.95]
    pub delay_wet:  f32,  // delay mix               [0.0 – 1.0]
    pub room_size:  f32,  // reverb tail length       [0.0 – 0.98]
    pub reverb_wet: f32,  // reverb mix               [0.0 – 1.0]
}

//==========Instrument==========
// Complete sound bundle. Built by each instrument config file, consumed by InstrumentEngine.

#[derive(Debug, Clone, Copy)]
pub struct Instrument
{
    pub kind:     InstrumentKind,
    pub waveform: Waveform,
    pub envelope: EnvelopeParams,
    pub filter:   FilterPreset,
    pub effects:  EffectConfig,
}

impl Instrument
{
    // Takes an InstrumentKind. Calls the matching config file and returns the bundle.
    pub fn get(kind: InstrumentKind) -> Self
    {
        match kind
        {
            InstrumentKind::Piano => super::config::piano::config(),
            InstrumentKind::Flute => super::config::flute::config(),
            InstrumentKind::Bass  => super::config::bass::config(),
            InstrumentKind::Pad   => super::config::pad::config(),
            InstrumentKind::Lead  => super::config::lead::config(),
        }
    }
}

//==========InstrumentEngine==========
// Owns the VoicePool + EffectsChain for one track.
// This is the only struct the sequencer and play.rs touch.

pub struct InstrumentEngine
{
    pub pool:    VoicePool,
    pub fx:      EffectsChain,
    pub current: InstrumentKind,
    sample_rate: u32,
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
        println!("[instrument] Loaded: {}", kind.name());

        Self { pool, fx, current: kind, sample_rate }
    }

    // Takes an instrument kind. Reloads config from the matching file and rebuilds everything.
    pub fn set_instrument(&mut self, kind: InstrumentKind)
    {
        let inst = Instrument::get(kind);
        self.current = kind;
        self.pool = VoicePool::new(self.sample_rate, inst.waveform, inst.envelope);
        self.pool.set_filter(inst.filter);
        self.fx = Self::build_fx(&inst.effects, self.sample_rate);
        println!("[instrument] Switched to: {}", kind.name());
    }

    // Takes nothing. Returns one audio sample: voices summed, then effects applied.
    // Called 44 100 times per second from the cpal callback.
    #[inline(always)]
    pub fn next_sample(&mut self) -> f32
    {
        let dry = self.pool.next_sample();
        self.fx.process(dry)
    }

    // Takes an EffectConfig and sample rate. Builds and returns the EffectsChain.
    fn build_fx(cfg: &EffectConfig, sample_rate: u32) -> EffectsChain
    {
        use super::effects::{Delay, Reverb};
        EffectsChain
        {
            delay:  Delay::new(cfg.delay_ms, cfg.feedback, cfg.delay_wet, sample_rate),
            reverb: Reverb::new(cfg.room_size, cfg.reverb_wet),
        }
    }
}
