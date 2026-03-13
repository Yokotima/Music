/// Signal chain per voice:
///   Oscillator → Filter → Envelope (amplitude) → output

use super::envelope::{
    Envelope,
    EnvelopeParams,
    EnvelopeState
};
use super::filter::{
    BiquadFilter,
    FilterPreset
};
use super::oscillator::{
    Oscillator,
    Waveform
};

pub const MAX_VOICES: usize = 32;
const MASTER_GAIN: f32 = 1.0 / (MAX_VOICES as f32 / 4.0);

// Takes a MIDI note number [0-127]. Returns the frequency in Hz.
pub fn midi_to_freq(note: u8) -> f32
{
    440.0 * 2.0_f32.powf((note as f32 - 69.0) / 12.0)
}

//==========Voice==========
pub struct Voice
{
    pub oscillator:Oscillator,
    pub filter:BiquadFilter,
    pub envelope:Envelope,
    pub note:u8,
    pub velocity:f32,
    pub age:u32,
    pub active:bool,
}

impl Voice 
{
    // Takes a sample rate and waveform. Returns a silent voice ready to be triggered.
    fn new(sample_rate: u32, waveform: Waveform) -> Self
    {
        Self
        {
            oscillator:Oscillator::new(440.0, sample_rate, waveform),
            filter:BiquadFilter::new(sample_rate),
            envelope:Envelope::new(sample_rate),
            note:255,
            velocity:0.0,
            age:0,
            active:false,
        }
    }

    // Takes a MIDI note, velocity and ADSR params. Starts playing — sets frequency and triggers Attack.
    fn note_on(&mut self, note: u8, velocity: f32, params: &EnvelopeParams)
    {
        let freq = midi_to_freq(note);
        self.oscillator.set_frequency(freq);
        self.filter.reset();   // clear delay lines to avoid residual noise
        self.envelope.note_on(params);
        self.note = note;
        self.velocity = velocity.clamp(0.0, 1.0);
        self.age = 0;
        self.active = true;
    }

    // Takes ADSR params. Triggers the Release phase — the voice fades out naturally.
    fn note_off(&mut self, params: &EnvelopeParams)
    {
        self.envelope.note_off(params);
    }

    /// Signal chain: Oscillator → Filter → Envelope
    // Takes ADSR params. Returns one audio sample: oscillator shaped by filter and envelope.
    // Returns 0.0 immediately if the voice is inactive.
    #[inline(always)]
    fn next_sample(&mut self, params: &EnvelopeParams) -> f32
    {
        if !self.active
        {
            return 0.0;
        }

        let osc_out = self.oscillator.next_sample();
        let filtered_out = self.filter.process(osc_out);  // ← filter in chain
        let amp = self.envelope.next_sample(params);
        self.age += 1;

        if self.envelope.is_idle()
        {
            self.active = false;
            self.note   = 255;
        }
        let vel_amp = self.velocity * self.velocity;
        let out = filtered_out * amp * vel_amp;

        out.tanh()
    }
}

//==========VoicePool==========
pub struct VoicePool
{
    voices:Vec<Voice>,
    pub params:EnvelopeParams,
}

impl VoicePool
{
    // Takes sample rate, waveform and ADSR params. Returns a pool of 32 pre-allocated voices.
    pub fn new(sample_rate: u32, waveform: Waveform, params: EnvelopeParams) -> Self
    {
        let voices = (0..MAX_VOICES)
            .map(|_| Voice::new(sample_rate, waveform))
            .collect();
        Self
        { 
            voices,
            params,
        }
    }

    /// Apply a filter preset to all voices.
    /// Call this when switching instruments.
    // Takes a filter preset. Applies it to all 32 voices (type, cutoff, resonance).
    pub fn set_filter(&mut self, preset: FilterPreset)
    {
        for v in self.voices.iter_mut()
        {
            v.filter.set_params(preset.filter_type, preset.cutoff_hz, preset.q);
        }
    }

    // Takes a MIDI note and velocity. Finds a free voice (or steals one) and starts playing.
    pub fn note_on(&mut self, note: u8, velocity: f32)
    {
        if let Some(v) = self.voices.iter_mut().find(|v| v.active && v.note == note)
        {
            v.note_on(note, velocity, &self.params);
            return;
        }
        let idx = self.find_free_voice().unwrap_or_else(|| self.steal_voice());
        self.voices[idx].note_on(note, velocity, &self.params);
    }

    // Takes a MIDI note. Triggers the Release phase on the voice playing that note.
    pub fn note_off(&mut self, note: u8)
    {
        for v in self.voices.iter_mut()
        {
            if v.active && v.note == note
            {
                v.note_off(&self.params);
            }
        }
    }

    // Takes nothing. Returns one audio sample — the sum of all active voices.
    #[inline(always)]
    pub fn next_sample(&mut self) -> f32
    {
        let mut sum = 0.0_f32;
        for voice in self.voices.iter_mut()
        {
            sum += voice.next_sample(&self.params);
        }
        sum * MASTER_GAIN
    }

    // Takes nothing. Returns how many voices are currently active (playing or in release).
    pub fn active_voice_count(&self) -> usize
    {
        self.voices.iter().filter(|v| v.active).count()
    }

    // Takes nothing. Returns the index of the first idle voice, or None if all are busy.
    fn find_free_voice(&self) -> Option<usize>
    {
        self.voices.iter().position(|v| !v.active)
    }

    // Takes nothing. Returns the index of the best voice to steal when the pool is full.
    // Priority: voices in Release first, then the oldest active voice.
    fn steal_voice(&self) -> usize
    {
        if let Some(idx) = self.voices.iter().position(|v| {v.active && v.envelope.state() == EnvelopeState::Release}) 
        {
            return idx;
        }
        self.voices.iter().enumerate().filter(|(_, v)| v.active).max_by_key(|(_, v)| v.age).map(|(i, _)| i).unwrap_or(0)
    }
}
