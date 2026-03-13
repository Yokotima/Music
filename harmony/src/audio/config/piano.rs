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

        waveform: Waveform::Triangle,

        envelope: EnvelopeParams
        {
            attack_secs: 0.002,
            decay_secs: 1.8, 
            sustain_level: 0.0,
            release_secs: 0.3,
        },

        filter: FilterPreset
        {
            filter_type: FilterType::LowPass,
            cutoff_hz: 4_000.0,
            q: 0.707, 
        },

        effects: EffectConfig
        {
            delay_ms: 0.0,
            feedback: 0.0,
            delay_wet: 0.0,
            room_size: 0.45,
            reverb_wet: 0.08,
        },
    }
}
