extern crate serde;
extern crate serde_json;

use serde::{Deserialize, Serialize};
use std::fs;

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

pub fn save_to_json(project: &Project, path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let data = serde_json::to_string_pretty(project)?;
    fs::write(path, data)?;
    Ok(())
}
