mod files;
mod ui;
mod utils;
mod audio;
mod sequencer;

use anyhow::Result;
use std::io::{self, BufRead};

fn main() -> Result<()>
{
    ui::window::window();
    Ok(())
}
