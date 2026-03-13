
///
/// Step Sequencer — intégration avec InstrumentEngine
///
/// ## Architecture
///
/// ```
/// StepSequencer
/// ├── tracks: Vec<Track>        — une ligne par instrument actif
/// │   └── Track
/// │       ├── engine: InstrumentEngine   — synthé dédié à cette piste
/// │       ├── steps: Vec<Step>           — grille on/off, note, vélocité
/// │       └── default_note: u8           — note MIDI jouée sur chaque step actif
/// ├── bpm: f32
/// ├── step_count: usize         — nombre de pas (1–64, défaut 16)
/// ├── current_step: usize       — pas en cours de lecture
/// ├── sample_clock: u32         — échantillons écoulés depuis le début du step actuel
/// └── samples_per_step: u32     — recalculé à chaque changement de BPM
/// ```
///
/// ## Intégration cpal
///
/// `StepSequencer::next_sample()` est la seule fonction à appeler
/// depuis le callback audio. Elle :
///   1. Avance l'horloge interne
///   2. Déclenche note_on / note_off au bon moment sur chaque piste
///   3. Somme et retourne un échantillon f32 prêt à écrire dans le buffer cpal
///
/// ## Exemple d'utilisation (dans audio_engine.rs)
///
/// ```rust
/// let mut seq = StepSequencer::new(120.0, 16, SAMPLE_RATE);
///
/// // Piste 0 — kick sur les temps 0, 4, 8, 12
/// seq.add_track(InstrumentKind::Bass, 36);         // C2 = kick MIDI convention
/// seq.set_step(0, 0, true);
/// seq.set_step(0, 4, true);
/// seq.set_step(0, 8, true);
/// seq.set_step(0, 12, true);
///
/// // Piste 1 — mélodie lead
/// seq.add_track(InstrumentKind::Lead, 60);         // C4
/// seq.set_step(1, 0, true);
/// seq.set_step_note(1, 4, 64u8);                   // E4 sur le step 4
/// seq.set_step(1, 8, true);
///
/// // Dans le callback cpal :
/// let sample = seq.next_sample();
/// ```
use crate::audio::instruments::{InstrumentEngine, InstrumentKind};

// ═══════════════════════════════════════════════════════
// Constantes
// ═══════════════════════════════════════════════════════

/// Durée d'une note comme fraction d'un step.
/// 0.8 = la note dure 80 % du step, laissant 20 % de silence entre les pas.
/// Cela évite que deux notes identiques consécutives se fondent.
const NOTE_GATE: f32 = 0.8;

pub const DEFAULT_STEP_COUNT: usize = 16;
pub const MIN_STEPS: usize          = 1;
pub const MAX_STEPS: usize          = 64;
pub const MIN_BPM: f32              = 40.0;
pub const MAX_BPM: f32              = 300.0;

// ═══════════════════════════════════════════════════════
// Step — une case de la grille
// ═══════════════════════════════════════════════════════

/// Une case dans la grille du séquenceur.
#[derive(Debug, Clone, Copy)]
pub struct Step
{
    /// Case activée ou non.
    pub active: bool,

    /// Note MIDI jouée sur ce step [0–127].
    /// Si None, la piste utilise sa note par défaut.
    pub note: Option<u8>,

    /// Vélocité [0.0–1.0]. None = vélocité par défaut de la piste.
    pub velocity: Option<f32>,
}

impl Step
{
    fn off() -> Self
    {
        Self { active: false, note: None, velocity: None }
    }
}

// ═══════════════════════════════════════════════════════
// Track — une ligne de la grille
// ═══════════════════════════════════════════════════════

/// Une piste = un instrument + une grille de N steps.
pub struct Track
{
    /// Le synthé dédié à cette piste (voix, filtre, effets configurés).
    pub engine: InstrumentEngine,

    /// Grille de steps.
    pub steps: Vec<Step>,

    /// Note MIDI par défaut pour tous les steps sans note explicite.
    pub default_note: u8,

    /// Vélocité par défaut [0.0–1.0].
    pub default_velocity: f32,

    /// Note en cours de lecture sur cette piste (pour émettre le note_off).
    active_note: Option<u8>,

    /// Piste muette (les steps sont ignorés).
    pub muted: bool,
}

impl Track
{
    // Crée une piste pour un instrument donné avec N steps vides.
    fn new(kind: InstrumentKind, step_count: usize, default_note: u8, sample_rate: u32) -> Self
    {
        Self
        {
            engine:           InstrumentEngine::new(kind, sample_rate),
            steps:            vec![Step::off(); step_count],
            default_note,
            default_velocity: 0.8,
            active_note:      None,
            muted:            false,
        }
    }

    // Redimensionne la grille (allonge avec des steps vides ou tronque).
    fn resize(&mut self, new_count: usize)
    {
        self.steps.resize(new_count, Step::off());
    }
}

// ═══════════════════════════════════════════════════════
// StepSequencer
// ═══════════════════════════════════════════════════════

/// Séquenceur à pas — gère N pistes et avance automatiquement dans le callback audio.
pub struct StepSequencer
{
    /// Toutes les pistes du séquenceur.
    pub tracks: Vec<Track>,

    /// Tempo en battements par minute.
    bpm: f32,

    /// Nombre de pas dans la grille (identique pour toutes les pistes).
    pub step_count: usize,

    /// Pas courant [0, step_count).
    pub current_step: usize,

    /// Échantillons écoulés depuis le début du pas courant.
    sample_clock: u32,

    /// Nombre d'échantillons par pas (recalculé si BPM ou step_count change).
    /// Un pas = une double-croche à 4/4 quand step_count = 16.
    samples_per_step: u32,

    /// Fréquence d'échantillonnage (Hz).
    sample_rate: u32,

    /// Le séquenceur tourne-t-il ?
    pub playing: bool,

    /// Lecture en boucle (true par défaut).
    pub looping: bool,
}

impl StepSequencer
{
    // ── Construction ──────────────────────────────────────────────────

    /// Crée un séquenceur vide.
    ///
    /// # Arguments
    /// * `bpm`         — tempo initial
    /// * `step_count`  — nombre de pas (1–64)
    /// * `sample_rate` — fréquence d'échantillonnage du moteur audio
    pub fn new(bpm: f32, step_count: usize, sample_rate: u32) -> Self
    {
        let step_count = step_count.clamp(MIN_STEPS, MAX_STEPS);
        let bpm        = bpm.clamp(MIN_BPM, MAX_BPM);

        Self
        {
            tracks:           Vec::new(),
            bpm,
            step_count,
            current_step:     0,
            sample_clock:     0,
            samples_per_step: Self::calc_samples_per_step(bpm, step_count, sample_rate),
            sample_rate,
            playing:          false,
            looping:          true,
        }
    }

    // Calcule le nombre d'échantillons pour un pas.
    //
    // Un step = une double-croche dans une grille 16 steps / 4 beats (4/4).
    // Formule : samples = (60 / BPM) * sample_rate / (step_count / 4)
    //         = 240 * sample_rate / (BPM * step_count)
    // Exemple : 120 BPM, 16 steps, 44100 Hz → 5512.5 → 5513 samples/step
    fn calc_samples_per_step(bpm: f32, step_count: usize, sample_rate: u32) -> u32
    {
        let steps_per_beat = step_count as f32 / 4.0;
        let beat_samples   = sample_rate as f32 * 60.0 / bpm;
        (beat_samples / steps_per_beat).round() as u32
    }

    // ── Gestion des pistes ────────────────────────────────────────────

    /// Ajoute une piste pour un instrument donné.
    ///
    /// # Arguments
    /// * `kind`         — type d'instrument
    /// * `default_note` — note MIDI jouée par défaut (ex : 36 = kick, 60 = C4)
    ///
    /// # Returns
    /// Index de la piste créée.
    pub fn add_track(&mut self, kind: InstrumentKind, default_note: u8) -> usize
    {
        let track = Track::new(kind, self.step_count, default_note, self.sample_rate);
        self.tracks.push(track);
        self.tracks.len() - 1
    }

    /// Supprime une piste par index. Coupe d'abord toute note active.
    pub fn remove_track(&mut self, track_idx: usize)
    {
        if track_idx < self.tracks.len()
        {
            if let Some(note) = self.tracks[track_idx].active_note
            {
                self.tracks[track_idx].engine.pool.note_off(note);
            }
            self.tracks.remove(track_idx);
        }
    }

    // ── Modification de la grille ────────────────────────────────────

    /// Active ou désactive un step.
    /// Les indices invalides sont ignorés silencieusement.
    pub fn set_step(&mut self, track_idx: usize, step_idx: usize, active: bool)
    {
        if let Some(track) = self.tracks.get_mut(track_idx)
        {
            if let Some(step) = track.steps.get_mut(step_idx)
            {
                step.active = active;
            }
        }
    }

    /// Change la note MIDI d'un step spécifique et l'active automatiquement.
    /// Passe `None` pour revenir à la note par défaut de la piste.
    pub fn set_step_note(&mut self, track_idx: usize, step_idx: usize, note: impl Into<Option<u8>>)
    {
        if let Some(track) = self.tracks.get_mut(track_idx)
        {
            if let Some(step) = track.steps.get_mut(step_idx)
            {
                step.note   = note.into();
                step.active = true; // assigner une note active le step
            }
        }
    }

    /// Change la vélocité d'un step spécifique [0.0–1.0].
    pub fn set_step_velocity(&mut self, track_idx: usize, step_idx: usize, velocity: f32)
    {
        if let Some(track) = self.tracks.get_mut(track_idx)
        {
            if let Some(step) = track.steps.get_mut(step_idx)
            {
                step.velocity = Some(velocity.clamp(0.0, 1.0));
            }
        }
    }

    /// Efface tous les steps d'une piste (tous à off).
    pub fn clear_track(&mut self, track_idx: usize)
    {
        if let Some(track) = self.tracks.get_mut(track_idx)
        {
            for step in track.steps.iter_mut() { *step = Step::off(); }
        }
    }

    /// Efface toutes les pistes.
    pub fn clear_all(&mut self)
    {
        for i in 0..self.tracks.len() { self.clear_track(i); }
    }

    // ── Paramètres de transport ──────────────────────────────────────

    /// Change le BPM à la volée (prend effet dès le prochain step).
    pub fn set_bpm(&mut self, bpm: f32)
    {
        self.bpm             = bpm.clamp(MIN_BPM, MAX_BPM);
        self.samples_per_step = Self::calc_samples_per_step(self.bpm, self.step_count, self.sample_rate);
    }

    /// Retourne le BPM courant.
    pub fn bpm(&self) -> f32 { self.bpm }

    /// Change le nombre de pas et redimensionne toutes les pistes.
    pub fn set_step_count(&mut self, count: usize)
    {
        let count        = count.clamp(MIN_STEPS, MAX_STEPS);
        self.step_count  = count;
        for track in self.tracks.iter_mut() { track.resize(count); }
        self.samples_per_step = Self::calc_samples_per_step(self.bpm, count, self.sample_rate);
        self.current_step     = self.current_step.min(count - 1);
    }

    /// Lance la lecture.
    pub fn play(&mut self)  { self.playing = true; }

    /// Met en pause (conserve la position).
    pub fn pause(&mut self)
    {
        self.playing = false;
        self.all_notes_off();
    }

    /// Stop et retour au début.
    pub fn stop(&mut self)
    {
        self.playing      = false;
        self.current_step = 0;
        self.sample_clock = 0;
        self.all_notes_off();
    }

    /// Retourne la position de lecture courante (step, sample dans le step).
    pub fn position(&self) -> (usize, u32)
    {
        (self.current_step, self.sample_clock)
    }

    /// Coupe toutes les notes sur toutes les pistes.
    /// Appelé sur pause/stop pour éviter les notes bloquées.
    pub fn all_notes_off(&mut self)
    {
        for track in self.tracks.iter_mut()
        {
            if let Some(note) = track.active_note.take()
            {
                track.engine.pool.note_off(note);
            }
        }
    }

    /// Nombre de pistes non muettes.
    pub fn active_track_count(&self) -> usize
    {
        self.tracks.iter().filter(|t| !t.muted).count()
    }

    // ── Moteur audio — appelé 44 100×/sec ───────────────────────────

    /// **Point d'entrée principal depuis le callback cpal.**
    ///
    /// Avance l'horloge interne, déclenche les notes et retourne
    /// un échantillon f32 qui est la somme de toutes les pistes.
    ///
    /// À appeler exactement une fois par échantillon dans la boucle cpal.
    #[inline(always)]
    pub fn next_sample(&mut self) -> f32
    {
        if self.playing
        {
            // ── Début d'un nouveau step : note_on ────────────────────
            if self.sample_clock == 0
            {
                self.trigger_step();
            }

            // ── Gate off : note_off après NOTE_GATE * durée du step ──
            let gate_sample = (self.samples_per_step as f32 * NOTE_GATE) as u32;
            if self.sample_clock == gate_sample
            {
                self.gate_off_all();
            }

            // ── Avance l'horloge ─────────────────────────────────────
            self.sample_clock += 1;

            if self.sample_clock >= self.samples_per_step
            {
                self.sample_clock  = 0;
                self.current_step += 1;

                if self.current_step >= self.step_count
                {
                    if self.looping
                    {
                        self.current_step = 0;
                    }
                    else
                    {
                        self.current_step = self.step_count - 1;
                        self.playing      = false;
                    }
                }
            }
        }

        // Même à l'arrêt on laisse les enveloppes finir (release naturelle)
        self.sum_voices()
    }

    // ── Privé ────────────────────────────────────────────────────────

    // Déclenche note_on sur toutes les pistes actives pour le step courant.
    #[inline(always)]
    fn trigger_step(&mut self)
    {
        let step_idx = self.current_step;

        for track in self.tracks.iter_mut()
        {
            if track.muted { continue; }

            let step = track.steps[step_idx];

            // Couper la note précédente si toujours active
            if let Some(prev) = track.active_note.take()
            {
                track.engine.pool.note_off(prev);
            }

            if step.active
            {
                let note     = step.note.unwrap_or(track.default_note);
                let velocity = step.velocity.unwrap_or(track.default_velocity);
                track.engine.pool.note_on(note, velocity);
                track.active_note = Some(note);
            }
        }
    }

    // Déclenche note_off sur toutes les pistes (fin de gate).
    #[inline(always)]
    fn gate_off_all(&mut self)
    {
        for track in self.tracks.iter_mut()
        {
            if let Some(note) = track.active_note.take()
            {
                track.engine.pool.note_off(note);
            }
        }
    }

    // Somme les échantillons de toutes les pistes.
    #[inline(always)]
    fn sum_voices(&mut self) -> f32
    {
        let mut sum = 0.0_f32;
        for track in self.tracks.iter_mut()
        {
            sum += track.engine.next_sample();
        }
        sum
    }
}

// ═══════════════════════════════════════════════════════
// Tests unitaires
// ═══════════════════════════════════════════════════════

#[cfg(test)]
mod tests
{
    use super::*;

    const SR: u32 = 44_100;

    #[test]
    fn test_samples_per_step_120bpm_16steps()
    {
        // 240 / (120 * 16) * 44100 = 5512.5 → 5513
        let sps = StepSequencer::calc_samples_per_step(120.0, 16, SR);
        assert!((sps as i32 - 5513).abs() <= 1, "sps={sps}");
    }

    #[test]
    fn test_add_track_returns_correct_index()
    {
        let mut seq = StepSequencer::new(120.0, 16, SR);
        assert_eq!(seq.add_track(InstrumentKind::Piano, 60), 0);
        assert_eq!(seq.add_track(InstrumentKind::Bass,  36), 1);
    }

    #[test]
    fn test_set_step_toggle()
    {
        let mut seq = StepSequencer::new(120.0, 16, SR);
        seq.add_track(InstrumentKind::Lead, 60);
        seq.set_step(0, 3, true);
        assert!(seq.tracks[0].steps[3].active);
        seq.set_step(0, 3, false);
        assert!(!seq.tracks[0].steps[3].active);
    }

    #[test]
    fn test_set_step_note_activates_step()
    {
        let mut seq = StepSequencer::new(120.0, 16, SR);
        seq.add_track(InstrumentKind::Lead, 60);
        seq.set_step_note(0, 5, 64u8);
        assert!(seq.tracks[0].steps[5].active);
        assert_eq!(seq.tracks[0].steps[5].note, Some(64));
    }

    #[test]
    fn test_clear_track_resets_all_steps()
    {
        let mut seq = StepSequencer::new(120.0, 16, SR);
        seq.add_track(InstrumentKind::Piano, 60);
        for i in 0..16 { seq.set_step(0, i, true); }
        seq.clear_track(0);
        assert!(seq.tracks[0].steps.iter().all(|s| !s.active));
    }

    #[test]
    fn test_looping_wraps_current_step()
    {
        let mut seq = StepSequencer::new(120.0, 4, SR);
        seq.add_track(InstrumentKind::Piano, 60);
        seq.play();
        let sps = seq.samples_per_step as usize;
        for _ in 0..(sps * 4 + 1) { seq.next_sample(); }
        assert_eq!(seq.current_step, 0);
    }

    #[test]
    fn test_stop_resets_position()
    {
        let mut seq = StepSequencer::new(120.0, 16, SR);
        seq.add_track(InstrumentKind::Piano, 60);
        seq.play();
        let sps = seq.samples_per_step as usize;
        for _ in 0..sps * 5 { seq.next_sample(); }
        assert!(seq.current_step > 0);
        seq.stop();
        assert_eq!(seq.current_step, 0);
        assert_eq!(seq.sample_clock, 0);
        assert!(!seq.playing);
    }

    #[test]
    fn test_next_sample_no_panic_empty_grid()
    {
        let mut seq = StepSequencer::new(120.0, 16, SR);
        seq.play();
        for _ in 0..1024 { let _ = seq.next_sample(); }
    }

    #[test]
    fn test_set_step_count_resizes_all_tracks()
    {
        let mut seq = StepSequencer::new(120.0, 16, SR);
        seq.add_track(InstrumentKind::Piano, 60);
        seq.add_track(InstrumentKind::Bass,  36);
        seq.set_step_count(32);
        assert_eq!(seq.tracks[0].steps.len(), 32);
        assert_eq!(seq.tracks[1].steps.len(), 32);
    }

    #[test]
    fn test_produces_audio_with_active_step()
    {
        let mut seq = StepSequencer::new(120.0, 4, SR);
        seq.add_track(InstrumentKind::Piano, 60);
        seq.set_step(0, 0, true);
        seq.play();
        let has_audio = (0..500).any(|_| seq.next_sample().abs() > 1e-6);
        assert!(has_audio);
    }
}
