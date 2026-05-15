extern crate serde;
extern crate serde_json;

use serde::{Deserialize, Serialize};
use std::fs;
use hound;

use crate::sequencer::sequencer::StepSequencer;
use crate::audio::instruments::InstrumentKind;

#[derive(Serialize, Deserialize, Debug)]
pub struct Project {
    pub name: String,
    pub version: String,
    pub description: String,
    pub tracks: Vec<Track>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Track {
    pub engine: InstrumentKind,
    pub steps: Vec<Step>,
    pub default_note: u8,
    pub default_velocity: f32,
    pub muted: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Step {
    pub active: bool,
    pub note: Option<u8>,
    pub velocity: Option<f32>,
}

pub fn save_to_json(project: &Project, path: &str) -> Result<String, Box<dyn std::error::Error>> {
    if !path.ends_with(".json") 
    {
        return Err("Path must end with .json".into());
    }

    let data = serde_json::to_string_pretty(project)?;
    fs::write(path, data)?;
    Ok(path.into())
}

pub fn load_from_json(path: &str) -> Result<Project, Box<dyn std::error::Error>> {
    if !path.ends_with(".json") 
    {
        return Err("Path must end with .json".into());
    }

    let data = fs::read_to_string(path)?;
    let project: Project = serde_json::from_str(&data)?;
    Ok(project)
}

fn build_sequencer_from_project(project: &Project) -> StepSequencer {
    let mut seq = StepSequencer::new(1000.0, project.tracks.len(), 44100);

    for (_track_idx, track) in project.tracks.iter().enumerate() 
    {
        if track.muted 
        {
            continue;
        }

        let seq_track_idx = seq.add_track(track.engine, track.default_note);

        for (i, step) in track.steps.iter().enumerate() 
        {
            seq.set_step(seq_track_idx, i, step.active);

            if let Some(note) = step.note 
            {
                seq.set_step_note(seq_track_idx, i, note);
            }

            if let Some(vel) = step.velocity 
            {
                seq.set_step_velocity(seq_track_idx, i, vel);
            }
        }
    }

    seq.stop();
    seq.play();

    seq
}

pub fn export_to_wav(
    project: &Project,
    path: &str
) -> Result<String, Box<dyn std::error::Error>> {

    if !path.ends_with(".wav") 
    {
        return Err("Path must end with .wav".into());
    }

    let sample_rate = 44100;

    let spec = hound::WavSpec 
    {
        channels: 2,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create(path, spec)?;

    let mut seq = build_sequencer_from_project(project);

    seq.stop();
    seq.play();

    let max_steps = seq.step_count;

    let mut last_step = 0;
    let mut safety_counter = 0;
    let max_samples = sample_rate * 30;

    for _ in 0..max_samples 
    {

        let (l, r) = seq.next_sample();

        let (step, _) = seq.position();

        if step == 0 && last_step == max_steps - 1 
        {
            break;
        }

        last_step = step;
        safety_counter += 1;

        let left = (l.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        let right = (r.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;

        writer.write_sample(left)?;
        writer.write_sample(right)?;

        if safety_counter >= max_samples as usize 
        {
            break;
        }
    }

    writer.finalize()?;

    Ok(path.into())
}
