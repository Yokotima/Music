use super::super::instruments::{Instrument, InstrumentKind, EffectConfig};
use super::super::envelope::EnvelopeParams;
use super::super::filter::{FilterPreset, FilterType};
use super::super::oscillator::Waveform;

// Takes nothing. Returns the complete Bass sound configuration.
pub fn config() -> Instrument
{
    Instrument
    {
        kind: InstrumentKind::Bass,

        waveform: Waveform::Sawtooth,

        envelope: EnvelopeParams
        {
            attack_secs: 0.003,
            decay_secs: 0.08,
            sustain_level: 0.7,
            release_secs: 0.08,
        },

        filter: FilterPreset
        {
            filter_type: FilterType::LowPass,
            cutoff_hz: 800.0,
            q: 1.2,
        },

        effects: EffectConfig
        {
            delay_ms: 250.0,
            feedback: 0.0,  
            delay_wet: 0.0,
            room_size: 0.5,
            reverb_wet: 0.0, 
        },
    }
}
