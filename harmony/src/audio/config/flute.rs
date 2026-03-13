use super::super::instruments::{Instrument, InstrumentKind, EffectConfig};
use super::super::envelope::EnvelopeParams;
use super::super::filter::{FilterPreset, FilterType};
use super::super::oscillator::Waveform;

// Takes nothing. Returns the complete Flute sound configuration.
pub fn config() -> Instrument
{
    Instrument
    {
        kind: InstrumentKind::Flute,

        waveform: Waveform::Sine,

        envelope: EnvelopeParams
        {
            attack_secs: 0.08,
            decay_secs: 0.1, 
            sustain_level: 0.85,
            release_secs: 0.25,
        },

        filter: FilterPreset
        {
            filter_type: FilterType::LowPass,
            cutoff_hz: 2_500.0,
            q: 0.6,  
        },
        effects: EffectConfig

        {
            delay_ms: 0.0,  
            feedback: 0.0,  
            delay_wet: 0.0, 
            room_size: 0.6, 
            reverb_wet: 0.20,
        },
    }
}
