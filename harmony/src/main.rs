mod files {
    pub mod save_to_json;
}
use files::save_to_json::{Project, save_to_json,Track,Step};

fn main() {
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
}