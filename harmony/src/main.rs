mod files;
mod ui;
mod utils;
mod audio;
mod sequencer;

use anyhow::Result;
use std::env;

use files::save_to_json::{Project, save_to_json, load_from_json};

fn main() -> Result<()> {

    let args: Vec<String> = env::args().collect();

    match args.get(1).map(|s| s.as_str()) {

        Some("window") => {
            ui::window::window()?;
        }

        Some("json") => {
            let project = Project {
                name: "Harmony".into(),
                version: "0.1.0".into(),
                description: "A Rust project management tool".into(),
                tracks: vec![],
            };

            match save_to_json(&project, "../test.json") {
                Ok(_) => println!("Project saved to test.json"),
                Err(e) => eprintln!("Error saving project: {}", e),
            }

            match load_from_json("../test.json") {
                Ok(project) => println!("Project loaded from test.json: {:?}", project),
                Err(e) => eprintln!("Error loading project: {}", e),
            }
        }

        _ => {
            println!("Available commands:");
            println!("  cargo run window   -> open UI window");
            println!("  cargo run json     -> test JSON save/load");
        }
    }

    Ok(())
}
