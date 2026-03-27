use super::super::instruments::{Instrument, InstrumentKind, EffectConfig};
use super::super::envelope::EnvelopeParams;
use super::super::filter::{FilterPreset, FilterType};
use super::super::oscillator::Waveform;

// Takes nothing. Returns the complete Piano sound configuration.
pub fn config() -> Instrument
{
    Instrument
    {
        kind: InstrumentKind::Piano,

        waveform: Waveform::KarplusStrong,

        envelope: EnvelopeParams
        {
            attack_secs: 0.005,
            decay_secs: 2.5, 
            sustain_level: 0.0,
            release_secs: 0.4,
        },

        filter: FilterPreset
        {
            filter_type: FilterType::LowPass,
            cutoff_hz: 6000.0,
            q: 0.5, 
        },

        effects: EffectConfig
        {
            delay_ms: 375.0,
            feedback: 0.2,
            delay_wet: 0.1,
            room_size: 0.85,
            reverb_wet: 0.25,
        },
    }
}
