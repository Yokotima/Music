use super::super::instruments::{Instrument, InstrumentKind, EffectConfig};
use super::super::envelope::EnvelopeParams;
use super::super::filter::{FilterPreset, FilterType};
use super::super::oscillator::Waveform;

// Takes nothing. Returns the complete Lead sound configuration.
pub fn config() -> Instrument
{
    Instrument
    {
        kind: InstrumentKind::Lead,

        waveform: Waveform::Square,

        envelope: EnvelopeParams
        {
            attack_secs: 0.005, 
            decay_secs: 0.15,   
            sustain_level: 0.75,
            release_secs: 0.15, 
        },

        filter: FilterPreset
        {
            filter_type: FilterType::LowPass,
            cutoff_hz: 3_000.0,
            q: 3.0, 
        },

        effects: EffectConfig
        {
            delay_ms: 300.0, 
            feedback: 0.45,  
            delay_wet: 0.25, 
            room_size: 0.5,
            reverb_wet: 0.15,
        },
    }
}
