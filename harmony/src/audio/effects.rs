/// A delay effect stores the input signal in a circular buffer and mixes
/// it back with the dry signal after a set time. Feedback routes the
/// delayed signal back into the delay input, creating repeating echoes
/// that decay naturally.
///
///   x[n] ──┬──────────────────────────────────────────► dry
///           │         ┌─────────────────────┐
///           └─► (+) ──► circular buffer[D] ──┬──► wet
///                ▲                           │
///                └───────── * feedback ──────┘
///
///
/// "A combination of all-pass filters and delay lines to simulate the
/// acoustic persistence of a space."
///
/// The classic Schroeder reverberator (1962) uses:
///   - 4 parallel comb filters (feedback delay lines) — simulate early reflections
///   - 2 series all-pass filters                     — diffuse the sound
///
/// Each comb filter i has a different delay length D_i and the same
/// feedback coefficient g. The all-pass filters smooth the echo density.
///
/// This is simpler than modern reverbs (Freeverb, convolution) but
/// entirely appropriate for an academic project — it produces a
/// recognizable, musical reverb tail.

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

pub struct EffectsChain
{
    pub delay:  Delay,
    pub reverb: Reverb,
}

impl EffectsChain
{
    // Takes a sample rate. Returns a default effects chain with delay (375ms) and reverb (0.75).
    pub fn new(sample_rate: u32) -> Self
    {
        Self
        {
            delay:Delay::new(375.0, 0.4, 0.25, sample_rate),
            reverb:Reverb::new(0.75, 0.30),
        }
    }

    // Takes a sample rate. Returns a completely dry chain — no delay, no reverb.
    pub fn dry(sample_rate: u32) -> Self
    {
        Self
        {
            delay:Delay::new(375.0, 0.0, 0.0, sample_rate),
            reverb:Reverb::new(0.0,  0.0),
        }
    }

    // Takes one audio sample. Runs it through Delay then Reverb. Returns the processed sample.
    #[inline(always)]
    pub fn process(&mut self, input: (f32, f32)) -> (f32, f32)
    {
        let (in_l, in_r) = input;

        // Process Left channel
        let after_delay_l = self.delay.process(in_l);
        let out_l = self.reverb.process(after_delay_l);

        // Process Right channel
        let after_delay_r = self.delay.process(in_r);
        let out_r = self.reverb.process(after_delay_r);

        (out_l, out_r)
    }
}
