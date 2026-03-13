use super::super::instruments::{Instrument, InstrumentKind, EffectConfig};
use super::super::envelope::EnvelopeParams;
use super::super::filter::{FilterPreset, FilterType};
use super::super::oscillator::Waveform;

// Takes nothing. Returns the complete Pad sound configuration.
pub fn config() -> Instrument
{
    Instrument
    {
        kind: InstrumentKind::Pad,

        waveform: Waveform::Sawtooth,

        envelope: EnvelopeParams
        {
            attack_secs: 0.2, 
            decay_secs: 0.2,  
            sustain_level: 0.80,
            release_secs: 0.8,
        },

        filter: FilterPreset
        {
            filter_type: FilterType::LowPass,
            cutoff_hz: 1_200.0,
            q: 1.2, 
        },

        effects: EffectConfig
        {
            delay_ms: 375.0,  
            feedback: 0.30,   
            delay_wet: 0.15,  
            room_size: 0.65,  
            reverb_wet: 0.25, 
        },
    }
}
