extern crate serde;
extern crate serde_json;

use serde::{Deserialize, Serialize};
use std::fs;
use hound;
use std::f32::consts::PI;

#[derive(Serialize, Deserialize, Debug)]
pub struct Project {
    pub name: String,
    pub version: String,
    pub description: String,
    pub tracks: Vec<Track>,    
}

#[derive(Serialize, Deserialize, Debug)]
pub enum InstrumentKind {
    Piano,
    Flute,
    Bass,
    Pad,
    Lead,
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
    //le path doit se terminer par .json
    if !path.ends_with(".json") 
    {
        return Err("Path must end with .json (case sensitive)".into());
    }
    let data = serde_json::to_string_pretty(project)?;
    fs::write(path, data)?;
    Ok(path.into())
}

pub fn load_from_json(path: &str) -> Result<Project, Box<dyn std::error::Error>> {
    if !path.ends_with(".json") 
    {
        return Err("Path must end with .json (case sensitive)".into());
    }
    let data = fs::read_to_string(path)?;
    let project: Project = serde_json::from_str(&data)?;
    Ok(project)
}

pub fn export_to_wav(project: &Project, path: &str) -> Result<String, Box<dyn std::error::Error>> {
    //pour tt ce qui est formules et tt pour cette fonction est a verifier 
    //mais si ca marche c good alors
    
    if !path.ends_with(".wav") 
    {
        return Err("Path must end with .wav (case sensitive)".into());
    }
    let sample_rate = 44100;      //44100 => valeur par defaut jsp trop quoi  
    let spec = hound::WavSpec //mettre dcp jai juste mis ce quon ma conseille
    {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer = hound::WavWriter::create(path, spec)?;

    let step_duration = 0.5; 

    for track in &project.tracks
    {
        for step in &track.steps 
        {
            if !step.active 
            { 
                continue; 
            }

            if let Some(note) = step.note 
            {
                let freq = 440.0 * 2f32.powf((note as f32 - 69.0) / 12.0);
                let velocity = step.velocity.unwrap_or(track.default_velocity);

                let num_samples = (step_duration * sample_rate as f32) as usize;
                for i in 0..num_samples 
                {
                    let t = i as f32 / sample_rate as f32;
                    let sample = (velocity * i16::MAX as f32 * (2.0 * PI * freq * t).sin()) as i16;
                    writer.write_sample(sample)?;
                }
            }
        }
    }

    writer.finalize()?;
    Ok(path.into())
}