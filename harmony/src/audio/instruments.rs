/// Each instrument is a complete configuration bundle:
///   - Waveform      (oscillator shape)
///   - EnvelopeParams (ADSR)
///   - FilterPreset  (tonal character)
///   - EffectConfig  (delay + reverb amounts)
///
///   1. Piano   — percussive, bright, natural decay
///   2. Flute   — soft, breathy, slow attack
///   3. Bass    — deep, punchy, tight
///   4. Pad     — slow, lush, wide
///   5. Lead    — bright, resonant, expressive
///
/// ## Architecture
///
/// An `Instrument` struct holds all parameters. The `InstrumentEngine`
/// owns a `VoicePool` + `EffectsChain` and reconfigures them when
/// the instrument changes. This is the final form of the audio engine
/// before sequencer integration (Step 8).

use super::effects::EffectsChain;
use super::envelope::EnvelopeParams;
use super::filter::FilterPreset;
use super::oscillator::Waveform;
use super::voice::VoicePool;

// ─────────────────────────────────────────────────────────────────────────────
// InstrumentKind
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum InstrumentKind {
    Piano,
    Flute,
    Bass,
    Pad,
    Lead,
}

impl InstrumentKind {
    pub fn name(&self) -> &'static str {
        match self {
            Self::Piano => "Piano",
            Self::Flute => "Flute",
            Self::Bass  => "Bass",
            Self::Pad   => "Pad",
            Self::Lead  => "Lead",
        }
    }

    /// All instruments in order, useful for cycling in demos.
    pub fn all() -> [InstrumentKind; 5] {
        [Self::Piano, Self::Flute, Self::Bass, Self::Pad, Self::Lead]
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// EffectConfig — delay + reverb amounts per instrument
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub struct EffectConfig {
    pub delay_ms:   f32,
    pub feedback:   f32,
    pub delay_wet:  f32,
    pub room_size:  f32,
    pub reverb_wet: f32,
}

impl EffectConfig {
    pub fn none() -> Self {
        Self { delay_ms: 250.0, feedback: 0.0, delay_wet: 0.0,
               room_size: 0.5,  reverb_wet: 0.0 }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Instrument — full parameter bundle
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub struct Instrument {
    pub kind:     InstrumentKind,
    pub waveform: Waveform,
    pub envelope: EnvelopeParams,
    pub filter:   FilterPreset,
    pub effects:  EffectConfig,
}

impl Instrument {
    // ── 1. Piano ─────────────────────────────────────────────────────────
    //
    // Character: percussive attack, natural exponential decay, no sustain.
    // A piano key releases energy instantly then fades — modeled by a
    // fast attack, long decay, zero sustain ADSR.
    // Triangle wave is used (softer than sawtooth, avoids harshness).
    // Gentle low-pass cuts the highest partials for a natural tone.
    // No delay or reverb — piano has its own natural resonance.
    pub fn piano() -> Self {
        Self {
            kind:     InstrumentKind::Piano,
            waveform: Waveform::Triangle,
            envelope: EnvelopeParams {
                attack_secs:   0.002,
                decay_secs:    1.8,
                sustain_level: 0.0,
                release_secs:  0.4,
            },
            filter: FilterPreset::piano(),   // LP 4kHz Q=0.707
            effects: EffectConfig::none(),
        }
    }

    // ── 2. Flute ─────────────────────────────────────────────────────────
    //
    // Character: breathy, airy, gentle vibrato-like softness.
    // Sine wave with slight attack — the flute's tone is nearly pure
    // with soft onset from the breath.
    // Very soft LP filter removes any digital harshness.
    // Light reverb for spatial feel (flutes sound good in halls).
    pub fn flute() -> Self {
        Self {
            kind:     InstrumentKind::Flute,
            waveform: Waveform::Sine,
            envelope: EnvelopeParams {
                attack_secs:   0.08,
                decay_secs:    0.1,
                sustain_level: 0.85,
                release_secs:  0.25,
            },
            filter: FilterPreset::flute(),   // LP 2.5kHz Q=0.6
            effects: EffectConfig {
                delay_ms:   0.0,
                feedback:   0.0,
                delay_wet:  0.0,
                room_size:  0.6,
                reverb_wet: 0.20,
            },
        }
    }

    // ── 3. Bass ───────────────────────────────────────────────────────────
    //
    // Character: tight, punchy, deep. Sub-bass fundamental, no highs.
    // Sawtooth gives the richest harmonic content before the LP filter
    // strips everything above ~800 Hz, leaving a round bass tone.
    // Very fast attack (almost percussive), short decay to sustain,
    // fast release so bass notes don't blur into each other.
    // No reverb — bass in a mix needs to be tight and dry.
    pub fn bass() -> Self {
        Self {
            kind:     InstrumentKind::Bass,
            waveform: Waveform::Sawtooth,
            envelope: EnvelopeParams {
                attack_secs:   0.003,
                decay_secs:    0.08,
                sustain_level: 0.7,
                release_secs:  0.08,
            },
            filter: FilterPreset::bass(),    // LP 800Hz Q=1.2
            effects: EffectConfig::none(),
        }
    }

    // ── 4. Pad ────────────────────────────────────────────────────────────
    pub fn pad() -> Self {
        Self {
            kind:     InstrumentKind::Pad,
            waveform: Waveform::Sawtooth,
            envelope: EnvelopeParams {
                attack_secs:   0.2,   // reduced from 0.4 — faster response, less CPU
                decay_secs:    0.2,
                sustain_level: 0.80,
                release_secs:  0.8,   // reduced from 1.8 — voices freed sooner
            },
            filter: FilterPreset::pad(),
            effects: EffectConfig {
                delay_ms:   375.0,
                feedback:   0.30,
                delay_wet:  0.15,
                room_size:  0.65,     // reduced from 0.85 — less reverb tail = less CPU
                reverb_wet: 0.25,     // reduced from 0.40
            },
        }
    }

    // ── 5. Lead ───────────────────────────────────────────────────────────
    //
    // Character: bright, cutting, resonant. Classic monophonic synth lead.
    // Square wave has the characteristic hollow midrange of vintage leads.
    // Resonant LP filter (Q=3) adds character and presence —
    // this is the filter that makes synth leads recognizable.
    // Medium delay creates rhythmic echoes, light reverb for space.
    pub fn lead() -> Self {
        Self {
            kind:     InstrumentKind::Lead,
            waveform: Waveform::Square,
            envelope: EnvelopeParams {
                attack_secs:   0.005,
                decay_secs:    0.15,
                sustain_level: 0.75,
                release_secs:  0.15,
            },
            filter: FilterPreset::lead(),    // LP 3kHz Q=3.0
            effects: EffectConfig {
                delay_ms:   300.0,
                feedback:   0.45,
                delay_wet:  0.25,
                room_size:  0.5,
                reverb_wet: 0.15,
            },
        }
    }

    /// Look up an instrument by kind.
    pub fn get(kind: InstrumentKind) -> Self {
        match kind {
            InstrumentKind::Piano => Self::piano(),
            InstrumentKind::Flute => Self::flute(),
            InstrumentKind::Bass  => Self::bass(),
            InstrumentKind::Pad   => Self::pad(),
            InstrumentKind::Lead  => Self::lead(),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// InstrumentEngine — VoicePool + EffectsChain bound to one instrument
// ─────────────────────────────────────────────────────────────────────────────

/// The complete per-instrument audio engine.
///
/// This is the object the audio callback will use in Step 8.
/// Hugo's sequencer sends NoteEvents to it; it returns samples.
pub struct InstrumentEngine {
    pub pool:    VoicePool,
    pub fx:      EffectsChain,
    pub current: InstrumentKind,
    sample_rate: u32,
}

impl InstrumentEngine {
    pub fn new(kind: InstrumentKind, sample_rate: u32) -> Self {
        let inst = Instrument::get(kind);

        let mut pool = VoicePool::new(sample_rate, inst.waveform, inst.envelope);
        pool.set_filter(inst.filter);

        let fx = Self::build_fx(&inst.effects, sample_rate);

        println!("[instrument] Loaded: {}", kind.name());

        Self { pool, fx, current: kind, sample_rate }
    }

    /// Switch to a different instrument.
    /// Releases all active voices first to avoid orphaned notes.
    pub fn set_instrument(&mut self, kind: InstrumentKind) {
        let inst      = Instrument::get(kind);
        self.current  = kind;
        self.pool     = VoicePool::new(self.sample_rate, inst.waveform, inst.envelope);
        self.pool.set_filter(inst.filter);
        self.fx       = Self::build_fx(&inst.effects, self.sample_rate);
        println!("[instrument] Switched to: {}", kind.name());
    }

    /// Process one sample: sum voices → apply effects → return output.
    #[inline(always)]
    pub fn next_sample(&mut self) -> f32 {
        let dry = self.pool.next_sample();
        self.fx.process(dry)
    }

    fn build_fx(cfg: &EffectConfig, sample_rate: u32) -> EffectsChain {
        use super::effects::{Delay, Reverb};
        EffectsChain {
            delay:  Delay::new(cfg.delay_ms, cfg.feedback, cfg.delay_wet, sample_rate),
            reverb: Reverb::new(cfg.room_size, cfg.reverb_wet),
        }
    }
}
