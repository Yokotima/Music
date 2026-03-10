

use anyhow::{anyhow, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{BufferSize, SampleRate, StreamConfig};


use crate::sequencer::sequencer::StepSequencer;
use super::instruments::InstrumentKind;

pub const SAMPLE_RATE: u32    = 44_100;
pub const MAX_BUFFER_SIZE: u32 = 1024;

pub struct AudioEngine {
    _stream: cpal::Stream,
}

impl AudioEngine {
    pub fn start() -> Result<Self> {
        let host   = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| anyhow!("No output audio device found"))?;

        println!("[audio] Device : {}", device.name().unwrap_or("unknown".into()));

        let config = StreamConfig {
            channels:    2,
            sample_rate: SampleRate(SAMPLE_RATE),
            buffer_size: BufferSize::Fixed(MAX_BUFFER_SIZE),
        };

        println!(
            "[audio] Config : {} Hz | {} ch | buffer {} samples ({:.1} ms)",
            SAMPLE_RATE, config.channels, MAX_BUFFER_SIZE,
            MAX_BUFFER_SIZE as f32 / SAMPLE_RATE as f32 * 1000.0
        );

        // ══════════════════════════════════════════════════════════════
        // ② NOUVEAU : construction du séquenceur
        //    - 120 BPM, grille 16 steps (= 4 mesures de 4 doubles-croches)
        // ══════════════════════════════════════════════════════════════
        let mut seq = StepSequencer::new(120.0, 16, SAMPLE_RATE);

        // ── Piste 0 : Bass / kick ──────────────────────────────────
        // add_track(instrument, note_midi_par_défaut)
        // 36 = C2, convention kick en MIDI
        seq.add_track(InstrumentKind::Bass, 36);
        seq.set_step(0, 0,  true);   // temps 1
        seq.set_step(0, 4,  true);   // temps 2
        seq.set_step(0, 8,  true);   // temps 3
        seq.set_step(0, 12, true);   // temps 4

        // ── Piste 1 : Lead / mélodie ───────────────────────────────
        // set_step_note(piste, step, note) — active le step et lui assigne une note
        seq.add_track(InstrumentKind::Lead, 60); // C4 par défaut
        seq.set_step_note(1,  0, 60u8);  // C4
        seq.set_step_note(1,  2, 62u8);  // D4
        seq.set_step_note(1,  4, 64u8);  // E4
        seq.set_step_note(1,  6, 65u8);  // F4
        seq.set_step_note(1,  8, 67u8);  // G4
        seq.set_step_note(1, 12, 64u8);  // E4

        // ── Piste 2 : Pad / harmonie ───────────────────────────────
        seq.add_track(InstrumentKind::Pad, 60);
        seq.set_step(2, 0, true);    // note par défaut (C4)
        seq.set_step(2, 8, true);

        // ③ Lancer la lecture
        seq.play();

        println!("[audio] Séquenceur prêt — {} pistes, {} steps à {} BPM",
            seq.tracks.len(), seq.step_count, seq.bpm());

        // ④ Le séquenceur est déplacé dans le callback (move)
        let channels = config.channels as usize;

        let stream = device.build_output_stream(
            &config,
            move |output: &mut [f32], _: &cpal::OutputCallbackInfo| {
                for frame in output.chunks_mut(channels) {

                    // ⑤ UNE seule ligne remplace toute la logique timeline/cursor/piano
                    //    next_sample() avance l'horloge, déclenche les notes
                    //    et retourne la somme de toutes les pistes.
                    let out = seq.next_sample();

                    for ch in frame.iter_mut() {
                        *ch = out;
                    }
                }
            },
            |err| eprintln!("[audio] Stream error: {err}"),
            None,
        )?;

        stream.play()?;
        std::thread::sleep(std::time::Duration::from_millis(100));
        println!("[audio] Stream started ✓ — press ENTER to stop");

        Ok(Self { _stream: stream })
    }
}
