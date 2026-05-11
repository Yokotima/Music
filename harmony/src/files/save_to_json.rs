extern crate serde;
extern crate serde_json;

use serde::{Deserialize, Serialize};
//use core::num;
use std::fs;
use hound;
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

    let step_duration = 0.07; 
    let num_samples_per_step = (step_duration * sample_rate as f32) as usize;
    let twop = 2.0 * std::f32::consts::PI;
    let total_steps = project.tracks.iter().map(|t| t.steps.len()).max().unwrap_or(0);
    let total_samples = total_steps * num_samples_per_step;
    let mut buffer = vec![0.0f32; total_samples];

    for track in &project.tracks
    {
        if track.muted
        {
            continue;
        }
        let mut phase = 0.0;
        for (step_id, step) in track.steps.iter().enumerate()
        {
            if !step.active
            {
                continue;
            }
            if let Some(note) = step.note
            {
                let freq = 440.0 * 2f32.powf((note as f32 - 69.0) / 12.0); 
                let velocity = step.velocity.unwrap_or(track.default_velocity);
                for i in 0..num_samples_per_step 
                {
                    let sample_index = step_id * num_samples_per_step + i;
                    if sample_index >= buffer.len()
                    {
                        break;
                    } 
                    let t = i as f32 / sample_rate as f32;                 
                    buffer[sample_index] += velocity * (twop * freq * t + phase).sin();
                }
                phase += twop * freq * (num_samples_per_step as f32 / sample_rate as f32);                
            }
        }
    }
    
    //polyphonie

    let mamp = buffer.iter().map(|s| s.abs()).fold(0.0f32,f32::max).max(1.0);

    for sample in buffer
    {
        let s = (sample / mamp).clamp(-1.0, 1.0);
        writer.write_sample((s * i16::MAX as f32) as i16)?;
    }   

    writer.finalize()?;
    Ok(path.into())
}
