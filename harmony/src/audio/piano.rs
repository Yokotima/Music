/// All 88 keys of a standard piano as an enum.
/// Each variant knows its own MIDI number and frequency.
///
/// Usage:
///   play_sound(PianoNote::A4, 0.8, &mut engine);
///   stop_sound(PianoNote::A4, &mut engine);

use crate::sequencer::sequencer::StepSequencer;
use super::instruments::InstrumentKind;

// PianoNote — all 88 keys of a standard piano
// Naming: Note + Octave. Sharps written as "s" (Cs4 = C#4, Fs3 = F#3).
// Standard piano: A0 (key 1, MIDI 21) → C8 (key 88, MIDI 108)
// Reference: A4 = 440.0 Hz (key 49, MIDI 69)

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PianoNote
{
    //====Octave 0 (keys 1-3)
    A0,   // key  1 | MIDI 21 |  27.500 Hz
    As0,  // key  2 | MIDI 22 |  29.135 Hz
    B0,   // key  3 | MIDI 23 |  30.868 Hz

    //====Octave 1 (keys 4-15)
    C1,   // key  4 | MIDI 24 |  32.703 Hz
    Cs1,  // key  5 | MIDI 25 |  34.648 Hz
    D1,   // key  6 | MIDI 26 |  36.708 Hz
    Ds1,  // key  7 | MIDI 27 |  38.891 Hz
    E1,   // key  8 | MIDI 28 |  41.203 Hz
    F1,   // key  9 | MIDI 29 |  43.654 Hz
    Fs1,  // key 10 | MIDI 30 |  46.249 Hz
    G1,   // key 11 | MIDI 31 |  48.999 Hz
    Gs1,  // key 12 | MIDI 32 |  51.913 Hz
    A1,   // key 13 | MIDI 33 |  55.000 Hz
    As1,  // key 14 | MIDI 34 |  58.270 Hz
    B1,   // key 15 | MIDI 35 |  61.735 Hz

    //====Octave 2 (keys 16-27)
    C2,   // key 16 | MIDI 36 |  65.406 Hz
    Cs2,  // key 17 | MIDI 37 |  69.296 Hz
    D2,   // key 18 | MIDI 38 |  73.416 Hz
    Ds2,  // key 19 | MIDI 39 |  77.782 Hz
    E2,   // key 20 | MIDI 40 |  82.407 Hz
    F2,   // key 21 | MIDI 41 |  87.307 Hz
    Fs2,  // key 22 | MIDI 42 |  92.499 Hz
    G2,   // key 23 | MIDI 43 |  97.999 Hz
    Gs2,  // key 24 | MIDI 44 | 103.826 Hz
    A2,   // key 25 | MIDI 45 | 110.000 Hz
    As2,  // key 26 | MIDI 46 | 116.541 Hz
    B2,   // key 27 | MIDI 47 | 123.471 Hz

    //====Octave 3 (keys 28-39)
    C3,   // key 28 | MIDI 48 | 130.813 Hz
    Cs3,  // key 29 | MIDI 49 | 138.591 Hz
    D3,   // key 30 | MIDI 50 | 146.832 Hz
    Ds3,  // key 31 | MIDI 51 | 155.563 Hz
    E3,   // key 32 | MIDI 52 | 164.814 Hz
    F3,   // key 33 | MIDI 53 | 174.614 Hz
    Fs3,  // key 34 | MIDI 54 | 184.997 Hz
    G3,   // key 35 | MIDI 55 | 195.998 Hz
    Gs3,  // key 36 | MIDI 56 | 207.652 Hz
    A3,   // key 37 | MIDI 57 | 220.000 Hz
    As3,  // key 38 | MIDI 58 | 233.082 Hz
    B3,   // key 39 | MIDI 59 | 246.942 Hz

    //====Octave 4 (keys 40-51)
    C4,   // key 40 | MIDI 60 | 261.626 Hz
    Cs4,  // key 41 | MIDI 61 | 277.183 Hz
    D4,   // key 42 | MIDI 62 | 293.665 Hz
    Ds4,  // key 43 | MIDI 63 | 311.127 Hz
    E4,   // key 44 | MIDI 64 | 329.628 Hz
    F4,   // key 45 | MIDI 65 | 349.228 Hz
    Fs4,  // key 46 | MIDI 66 | 369.994 Hz
    G4,   // key 47 | MIDI 67 | 391.995 Hz
    Gs4,  // key 48 | MIDI 68 | 415.305 Hz
    A4,   // key 49 | MIDI 69 | 440.000 Hz
    As4,  // key 50 | MIDI 70 | 466.164 Hz
    B4,   // key 51 | MIDI 71 | 493.883 Hz

    //====Octave 5 (keys 52-63)
    C5,   // key 52 | MIDI 72 | 523.251 Hz
    Cs5,  // key 53 | MIDI 73 | 554.365 Hz
    D5,   // key 54 | MIDI 74 | 587.330 Hz
    Ds5,  // key 55 | MIDI 75 | 622.254 Hz
    E5,   // key 56 | MIDI 76 | 659.255 Hz
    F5,   // key 57 | MIDI 77 | 698.456 Hz
    Fs5,  // key 58 | MIDI 78 | 739.989 Hz
    G5,   // key 59 | MIDI 79 | 783.991 Hz
    Gs5,  // key 60 | MIDI 80 | 830.609 Hz
    A5,   // key 61 | MIDI 81 | 880.000 Hz
    As5,  // key 62 | MIDI 82 | 932.328 Hz
    B5,   // key 63 | MIDI 83 | 987.767 Hz

    //====Octave 6 (keys 64-75)
    C6,   // key 64 | MIDI 84 | 1046.502 Hz
    Cs6,  // key 65 | MIDI 85 | 1108.731 Hz
    D6,   // key 66 | MIDI 86 | 1174.659 Hz
    Ds6,  // key 67 | MIDI 87 | 1244.508 Hz
    E6,   // key 68 | MIDI 88 | 1318.510 Hz
    F6,   // key 69 | MIDI 89 | 1396.913 Hz
    Fs6,  // key 70 | MIDI 90 | 1479.978 Hz
    G6,   // key 71 | MIDI 91 | 1567.982 Hz
    Gs6,  // key 72 | MIDI 92 | 1661.219 Hz
    A6,   // key 73 | MIDI 93 | 1760.000 Hz
    As6,  // key 74 | MIDI 94 | 1864.655 Hz
    B6,   // key 75 | MIDI 95 | 1975.533 Hz

    //====Octave 7 (keys 76-87)
    C7,   // key 76 | MIDI  96 | 2093.005 Hz
    Cs7,  // key 77 | MIDI  97 | 2217.461 Hz
    D7,   // key 78 | MIDI  98 | 2349.318 Hz
    Ds7,  // key 79 | MIDI  99 | 2489.016 Hz
    E7,   // key 80 | MIDI 100 | 2637.020 Hz
    F7,   // key 81 | MIDI 101 | 2793.826 Hz
    Fs7,  // key 82 | MIDI 102 | 2959.955 Hz
    G7,   // key 83 | MIDI 103 | 3135.963 Hz
    Gs7,  // key 84 | MIDI 104 | 3322.438 Hz
    A7,   // key 85 | MIDI 105 | 3520.000 Hz
    As7,  // key 86 | MIDI 106 | 3729.310 Hz
    B7,   // key 87 | MIDI 107 | 3951.066 Hz

    //====Octave 8 (key 88)
    C8,   // key 88 | MIDI 108 | 4186.009 Hz
}

impl PianoNote
{
    // Takes nothing. Returns the MIDI note number for this key [21-108].
    pub fn midi(&self) -> u8
    {
        match self
        {
            Self::A0 => 21,
            Self::As0 => 22,
            Self::B0 => 23,
            Self::C1 => 24,
            Self::Cs1 => 25,
            Self::D1 => 26,
            Self::Ds1 => 27,
            Self::E1 => 28,
            Self::F1 => 29,
            Self::Fs1 => 30,
            Self::G1 => 31,
            Self::Gs1 => 32,
            Self::A1 => 33,
            Self::As1 => 34,
            Self::B1 => 35,
            Self::C2 => 36,
            Self::Cs2 => 37,
            Self::D2 => 38,
            Self::Ds2 => 39,
            Self::E2 => 40,
            Self::F2 => 41,
            Self::Fs2 => 42,
            Self::G2 => 43,
            Self::Gs2 => 44,
            Self::A2 => 45,
            Self::As2 => 46,
            Self::B2 => 47,
            Self::C3 => 48,
            Self::Cs3 => 49,
            Self::D3 => 50,
            Self::Ds3 => 51,
            Self::E3 => 52,
            Self::F3 => 53,
            Self::Fs3 => 54,
            Self::G3 => 55,
            Self::Gs3 => 56,
            Self::A3 => 57,
            Self::As3 => 58,
            Self::B3 => 59,
            Self::C4 => 60,
            Self::Cs4 => 61,
            Self::D4 => 62,
            Self::Ds4 => 63,
            Self::E4 => 64,
            Self::F4 => 65,
            Self::Fs4 => 66,
            Self::G4 => 67,
            Self::Gs4 => 68,
            Self::A4 => 69,
            Self::As4 => 70,
            Self::B4 => 71,
            Self::C5 => 72,
            Self::Cs5 => 73,
            Self::D5 => 74,
            Self::Ds5 => 75,
            Self::E5 => 76,
            Self::F5 => 77,
            Self::Fs5 => 78,
            Self::G5 => 79,
            Self::Gs5 => 80,
            Self::A5 => 81,
            Self::As5 => 82,
            Self::B5 => 83,
            Self::C6 => 84,
            Self::Cs6 => 85,
            Self::D6 => 86,
            Self::Ds6 => 87,
            Self::E6 => 88,
            Self::F6 => 89,
            Self::Fs6 => 90,
            Self::G6 => 91,
            Self::Gs6 => 92,
            Self::A6 => 93,
            Self::As6 => 94,
            Self::B6 => 95,
            Self::C7 => 96,
            Self::Cs7 => 97,
            Self::D7 => 98,
            Self::Ds7 => 99,
            Self::E7 => 100,
            Self::F7 => 101,
            Self::Fs7 => 102,
            Self::G7 => 103,
            Self::Gs7 => 104,
            Self::A7 => 105,
            Self::As7 => 106,
            Self::B7 => 107,
            Self::C8 => 108,
        }
    }

    // Takes nothing. Returns the exact frequency in Hz for this key.
    // Formula: 440.0 * 2^((midi - 69) / 12)  (equal temperament, A4 = 440 Hz)
    pub fn freq(&self) -> f32
    {
        440.0 * 2.0_f32.powf((self.midi() as f32 - 69.0) / 12.0)
    }

    // Takes nothing. Returns the display name of this key as a string.
    pub fn name(&self) -> &'static str
    {
        match self
        {
            Self::A0 => "A0",
            Self::As0 => "A#0",
            Self::B0 => "B0",
            Self::C1 => "C1",
            Self::Cs1 => "C#1",
            Self::D1 => "D1",
            Self::Ds1 => "D#1",
            Self::E1 => "E1",
            Self::F1 => "F1",
            Self::Fs1 => "F#1",
            Self::G1 => "G1",
            Self::Gs1 => "G#1",
            Self::A1 => "A1",
            Self::As1 => "A#1",
            Self::B1 => "B1",
            Self::C2 => "C2",
            Self::Cs2 => "C#2",
            Self::D2 => "D2",
            Self::Ds2 => "D#2",
            Self::E2 => "E2",
            Self::F2 => "F2",
            Self::Fs2 => "F#2",
            Self::G2 => "G2",
            Self::Gs2 => "G#2",
            Self::A2 => "A2",
            Self::As2 => "A#2",
            Self::B2 => "B2",
            Self::C3 => "C3",
            Self::Cs3 => "C#3",
            Self::D3 => "D3",
            Self::Ds3 => "D#3",
            Self::E3 => "E3",
            Self::F3 => "F3",
            Self::Fs3 => "F#3",
            Self::G3 => "G3",
            Self::Gs3 => "G#3",
            Self::A3 => "A3",
            Self::As3 => "A#3",
            Self::B3 => "B3",
            Self::C4 => "C4",
            Self::Cs4 => "C#4",
            Self::D4 => "D4",
            Self::Ds4 => "D#4",
            Self::E4 => "E4",
            Self::F4 => "F4",
            Self::Fs4 => "F#4",
            Self::G4 => "G4",
            Self::Gs4 => "G#4",
            Self::A4 => "A4",
            Self::As4 => "A#4",
            Self::B4 => "B4",
            Self::C5 => "C5",
            Self::Cs5 => "C#5",
            Self::D5 => "D5",
            Self::Ds5 => "D#5",
            Self::E5 => "E5",
            Self::F5 => "F5",
            Self::Fs5 => "F#5",
            Self::G5 => "G5",
            Self::Gs5 => "G#5",
            Self::A5 => "A5",
            Self::As5 => "A#5",
            Self::B5 => "B5",
            Self::C6 => "C6",
            Self::Cs6 => "C#6",
            Self::D6 => "D6",
            Self::Ds6 => "D#6", 
            Self::E6 => "E6",  
            Self::F6 => "F6",
            Self::Fs6 => "F#6", 
            Self::G6 => "G6",  
            Self::Gs6 => "G#6",
            Self::A6 => "A6",  
            Self::As6 => "A#6", 
            Self::B6 => "B6",
            Self::C7 => "C7",  
            Self::Cs7 => "C#7", 
            Self::D7 => "D7",
            Self::Ds7 => "D#7", 
            Self::E7 => "E7",  
            Self::F7 => "F7",
            Self::Fs7 => "F#7", 
            Self::G7 => "G7",  
            Self::Gs7 => "G#7",
            Self::A7 => "A7",  
            Self::As7 => "A#7", 
            Self::B7 => "B7",
            Self::C8 => "C8",
        }
    }
}

//==========play_sound / stop_sound==========

// Takes a PianoNote, a velocity and the sequencer.
// Finds the first Piano track and triggers the note on it.
// If no Piano track exists yet, creates one automatically.
pub fn play_sound(note: PianoNote, velocity: f32, seq: &mut StepSequencer)
{
    let idx = seq.tracks
        .iter()
        .position(|t| matches!(t.engine.current, InstrumentKind::Piano));

    let idx = match idx
    {
        Some(i) => i,
        None    => seq.add_track(InstrumentKind::Piano, note.midi()),
    };

    seq.tracks[idx].engine.pool.note_on(note.midi(), velocity);
    println!("[piano] ON  {} ({:.3} Hz)", note.name(), note.freq());
}

// Takes a PianoNote and the sequencer. Stops the note (triggers Release phase).
pub fn stop_sound(note: PianoNote, seq: &mut StepSequencer)
{
    let idx = seq.tracks
        .iter()
        .position(|t| matches!(t.engine.current, InstrumentKind::Piano));

    if let Some(idx) = idx
    {
        seq.tracks[idx].engine.pool.note_off(note.midi());
        println!("[piano] OFF {}", note.name());
    }
}
