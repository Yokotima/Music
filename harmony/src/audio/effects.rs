#![allow(dead_code)]
//==========Circular Buffer — shared by both effects==========
struct CircularBuffer
{
    data:Vec<f32>,
    write_head:usize,
    len:usize,
}

impl CircularBuffer
{
    // Takes a size in samples. Returns a circular buffer filled with silence.
    fn new(size_samples: usize) -> Self
    {
        Self
        {
            data:vec![0.0; size_samples],
            write_head:0,
            len:size_samples,
        }
    }

    // Takes one audio sample. Writes it at the current head position and advances the head.
    #[inline(always)]
    fn write(&mut self, sample: f32)
    {
        self.data[self.write_head] = sample;
        self.write_head = (self.write_head + 1) % self.len;
    }

    // Takes a delay in samples. Returns the sample that was written that many samples ago.
    #[inline(always)]
    fn read(&self, delay: usize) -> f32
    {
        let idx = (self.write_head + self.len - delay) % self.len;
        self.data[idx]
    }
}

//==========Delay Effect==========
const MAX_DELAY_SAMPLES: usize = 44_100 * 2;

pub struct Delay
{
    buffer:CircularBuffer,
    delay_samples:usize,
    feedback:f32,
    wet_mix:f32,
}

impl Delay
{
    // Takes delay time (ms), feedback, wet mix and sample rate. Returns a configured delay effect.
    pub fn new(delay_ms: f32, feedback: f32, wet_mix: f32, sample_rate: u32) -> Self
    {
        let delay_samples = ((delay_ms / 1000.0) * sample_rate as f32) as usize;
        let delay_samples = delay_samples.clamp(1, MAX_DELAY_SAMPLES);

        Self 
        {
            buffer:CircularBuffer::new(MAX_DELAY_SAMPLES),
            delay_samples,
            feedback:feedback.clamp(0.0, 0.95),
            wet_mix:wet_mix.clamp(0.0, 1.0),
        }
    }

    // Takes a delay time in ms. Updates how far back in time the echo reads.
    pub fn set_delay_ms(&mut self, delay_ms: f32, sample_rate: u32)
    {
        self.delay_samples = ((delay_ms / 1000.0) * sample_rate as f32) as usize;
        self.delay_samples = self.delay_samples.clamp(1, MAX_DELAY_SAMPLES);
    }

    // Takes a feedback value [0.0-0.95]. Controls how many echoes repeat before fading.
    pub fn set_feedback(&mut self, feedback: f32)
    {
        self.feedback = feedback.clamp(0.0, 0.95);
    }

    // Takes a wet mix [0.0-1.0]. Controls how loud the echoes are in the output.
    pub fn set_wet_mix(&mut self, wet: f32)
    {
        self.wet_mix = wet.clamp(0.0, 1.0);
    }

    // Takes one dry audio sample. Returns it blended with its delayed echo.
    #[inline(always)]
    pub fn process(&mut self, input: f32) -> f32
    {
        let delayed  = self.buffer.read(self.delay_samples);
        let feedback_sample = input + delayed * self.feedback;
        self.buffer.write(feedback_sample);

        let dry = input  * (1.0 - self.wet_mix);
        let wet = delayed * self.wet_mix;
        dry + wet
    }
}

//==========Reverb — Schroeder Network==========

const NUM_COMBS: usize = 4;
const NUM_ALLPASS: usize = 2;
//const COMB_DELAYS: [usize; NUM_COMBS] = [1557, 1617, 1491, 1422];
//const ALLPASS_DELAYS: [usize; NUM_ALLPASS] = [225, 556];
const COMB_DELAYS: [usize; NUM_COMBS] = [1601, 1861, 2351, 2503]; 
const ALLPASS_DELAYS: [usize; NUM_ALLPASS] = [557, 227];
const ALLPASS_FEEDBACK: f32 = 0.5;

struct CombFilter
{
    buffer:CircularBuffer,
    delay:usize,
    feedback:f32,
}

impl CombFilter
{
    // Takes delay length and feedback. Returns a comb filter simulating one reflection path.
    fn new(delay_samples: usize, feedback: f32) -> Self
    {
        Self
        {
            buffer:CircularBuffer::new(delay_samples + 1),
            delay:delay_samples,
            feedback,
        }
    }

    // Takes one audio sample. Returns it added to its delayed self — creates a repeating echo.
    #[inline(always)]
    fn process(&mut self, input: f32) -> f32
    {
        let delayed = self.buffer.read(self.delay);
        let output  = input + self.feedback * delayed;
        self.buffer.write(output);
        output
    }
}

struct AllPassFilter
{
    buffer:CircularBuffer,
    delay:usize,
    feedback:f32,
}

impl AllPassFilter
{
    // Takes delay length and feedback. Returns an all-pass filter for echo diffusion.
    fn new(delay_samples: usize, feedback: f32) -> Self
    {
        Self
        {
            buffer:CircularBuffer::new(delay_samples + 1),
            delay:delay_samples,
            feedback,
        }
    }

    // Takes one audio sample. Returns it phase-shifted — smooths echo density into a reverb tail.
    #[inline(always)]
    fn process(&mut self, input: f32) -> f32
    {
        let delayed = self.buffer.read(self.delay);
        let output = -self.feedback * input + delayed + self.feedback * delayed;
        self.buffer.write(input + self.feedback * delayed);
        output
    }
}

pub struct Reverb
{
    combs:[CombFilter;   NUM_COMBS],
    allpass:[AllPassFilter; NUM_ALLPASS],
    room_size:f32,
    wet_mix:f32,
}

impl Reverb
{
    // Takes room size [0.0-0.98] and wet mix [0.0-1.0]. Returns a configured Schroeder reverb.
    pub fn new(room_size: f32, wet_mix: f32) -> Self
    {
        let room_size = room_size.clamp(0.0, 0.98);
        Self
        {
            combs:
            [
                CombFilter::new(COMB_DELAYS[0], room_size),
                CombFilter::new(COMB_DELAYS[1], room_size),
                CombFilter::new(COMB_DELAYS[2], room_size),
                CombFilter::new(COMB_DELAYS[3], room_size),
            ],
            allpass:
            [
                AllPassFilter::new(ALLPASS_DELAYS[0], ALLPASS_FEEDBACK),
                AllPassFilter::new(ALLPASS_DELAYS[1], ALLPASS_FEEDBACK),
            ],
            room_size,
            wet_mix:wet_mix.clamp(0.0, 1.0),
        }
    }

    // Takes a room size [0.0-0.98]. Updates the reverb tail length on all comb filters.
    pub fn set_room_size(&mut self, room_size: f32)
    {
        self.room_size = room_size.clamp(0.0, 0.98);
        for comb in self.combs.iter_mut()
        {
            comb.feedback = self.room_size;
        }
    }

    // Takes a wet mix [0.0-1.0]. Controls how loud the reverb tail is in the output.
    pub fn set_wet_mix(&mut self, wet: f32)
    {
        self.wet_mix = wet.clamp(0.0, 1.0);
    }

    // Takes one dry audio sample. Returns it blended with the full Schroeder reverb tail.
    #[inline(always)]
    pub fn process(&mut self, input: f32) -> f32
    {
        let mut reverb_sum = 0.0_f32;
        for comb in self.combs.iter_mut()
        {
            reverb_sum += comb.process(input);
        }
        reverb_sum /= NUM_COMBS as f32;

        let mut diffused = reverb_sum;
        for ap in self.allpass.iter_mut()
        {
            diffused = ap.process(diffused);
        }

        input * (1.0 - self.wet_mix) + diffused * self.wet_mix
    }
}

//==========EffectsChain — convenience wrapper==========

// Which global effect is active.
// None  = dry signal only
// Reverb = Schroeder reverb
// Delay  = echo repeat
// Chorus = LFO-modulated delay (width, depth, rate)
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum EffectMode
{
    None,
    Reverb,
    Delay,
    Chorus,
}

//==========Chorus==========
//
// A chorus duplicates the signal and runs it through a short delay
// whose length is modulated by a low-frequency oscillator (LFO).
// The slightly pitch-shifted copy blended with the dry signal gives
// the characteristic "wide" and "shimmering" chorus sound.
//
//   dry ──┬────────────────────────────────────────────► out L
//         │   delay[LFO(t)]                              out R
//         └──► read(pos - mod) ──► wet ──► mix
//
// We use two LFOs offset by 90° to spread L and R differently.
pub struct Chorus
{
    buffer:     Vec<f32>,   // circular delay buffer (shared L/R — mono input)
    write_pos:  usize,
    sample_rate: u32,

    lfo_phase_l: f32,       // LFO phase for left channel  [0.0, 1.0)
    lfo_phase_r: f32,       // LFO phase for right channel (offset 90°)

    pub rate_hz:  f32,      // LFO speed  — how fast the pitch wobbles  (0.1–5.0 Hz)
    pub depth_ms: f32,      // LFO depth  — max delay modulation in ms  (1–20 ms)
    pub wet_mix:  f32,      // blend dry/wet                             [0.0–1.0]
}

impl Chorus
{
    // Takes sample rate. Returns a chorus with musical defaults.
    pub fn new(sample_rate: u32) -> Self
    {
        // Buffer large enough for max depth (20 ms) + base delay (25 ms)
        let buf_size = (sample_rate as f32 * 0.05) as usize + 4;
        Self
        {
            buffer:      vec![0.0; buf_size],
            write_pos:   0,
            sample_rate,
            lfo_phase_l: 0.0,
            lfo_phase_r: 0.25, // 90° offset → stereo spread
            rate_hz:     0.5,
            depth_ms:    8.0,
            wet_mix:     0.5,
        }
    }

    // Takes a mono input sample. Returns a stereo (L, R) chorus output.
    #[inline(always)]
    pub fn process(&mut self, input: f32) -> (f32, f32)
    {
        // Write input into the circular buffer
        self.buffer[self.write_pos] = input;

        let buf_len   = self.buffer.len();
        let sr        = self.sample_rate as f32;
        let base_ms   = 15.0_f32;               // centre delay (ms)
        let base_samp = (base_ms / 1000.0 * sr) as usize;

        // LFO modulation: sine wave scaled to depth in samples
        let depth_samp = (self.depth_ms / 1000.0 * sr) as f32;

        use std::f32::consts::TAU;
        let mod_l = (self.lfo_phase_l * TAU).sin() * depth_samp;
        let mod_r = (self.lfo_phase_r * TAU).sin() * depth_samp;

        let delay_l = (base_samp as f32 + mod_l).max(1.0) as usize;
        let delay_r = (base_samp as f32 + mod_r).max(1.0) as usize;
        let delay_l = delay_l.min(buf_len - 1);
        let delay_r = delay_r.min(buf_len - 1);

        // Read from the modulated positions
        let idx_l = (self.write_pos + buf_len - delay_l) % buf_len;
        let idx_r = (self.write_pos + buf_len - delay_r) % buf_len;
        let wet_l = self.buffer[idx_l];
        let wet_r = self.buffer[idx_r];

        // Advance write head and LFO phases
        self.write_pos = (self.write_pos + 1) % buf_len;

        let lfo_inc = self.rate_hz / sr;
        self.lfo_phase_l = (self.lfo_phase_l + lfo_inc).fract();
        self.lfo_phase_r = (self.lfo_phase_r + lfo_inc).fract();

        let dry = 1.0 - self.wet_mix;
        (
            input * dry + wet_l * self.wet_mix,
            input * dry + wet_r * self.wet_mix,
        )
    }
}

pub struct EffectsChain
{
    pub delay:   Delay,
    pub reverb:  Reverb,
    pub chorus:  Chorus,
    pub mode:    EffectMode,
    pub wet_mix: f32,       // global wet amount exposed to the UI [0.0–1.0]
}

impl EffectsChain
{
    // Takes a sample rate. Returns a default effects chain with delay (375ms) and reverb (0.75).
    pub fn new(sample_rate: u32) -> Self
    {
        Self
        {
            delay:   Delay::new(375.0, 0.4, 0.25, sample_rate),
            reverb:  Reverb::new(0.75, 0.30),
            chorus:  Chorus::new(sample_rate),
            mode:    EffectMode::None,
            wet_mix: 0.5,
        }
    }

    // Takes a sample rate. Returns a completely dry chain — no delay, no reverb.
    pub fn dry(sample_rate: u32) -> Self
    {
        let mut chain = Self::new(sample_rate);
        chain.mode    = EffectMode::None;
        chain.delay.set_wet_mix(0.0);
        chain.reverb.set_wet_mix(0.0);
        chain
    }

    // Applies the active effect mode with the current wet_mix to one stereo frame.
    // Called 44 100 times per second from the audio callback.
    #[inline(always)]
    pub fn process(&mut self, input: (f32, f32)) -> (f32, f32)
    {
        let (in_l, in_r) = input;

        match self.mode
        {
            EffectMode::None =>
            {
                (in_l, in_r)
            }

            EffectMode::Reverb =>
            {
                self.reverb.set_wet_mix(self.wet_mix);
                let out_l = self.reverb.process(in_l);
                let out_r = self.reverb.process(in_r);
                (out_l, out_r)
            }

            EffectMode::Delay =>
            {
                self.delay.set_wet_mix(self.wet_mix);
                let out_l = self.delay.process(in_l);
                let out_r = self.delay.process(in_r);
                (out_l, out_r)
            }

            EffectMode::Chorus =>
            {
                self.chorus.wet_mix = self.wet_mix;
                // Chorus takes mono average, returns stereo
                let mono = (in_l + in_r) * 0.5;
                self.chorus.process(mono)
            }
        }
    }
}
