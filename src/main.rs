mod library;
mod market_board_analysis;
mod universalis;

use crate::{library::Library, universalis::Universalis};
use std::{error::Error, time::Instant};

#[derive(PartialEq)]
pub enum RunMode {
    OnlyCustom,
    OnlyGathering,
    OnlyCrafting,
    All,
}

#[allow(dead_code)]
pub struct Settings {
    min_crafting_velocity: f32,
    listings_ratio: f32,
    min_profit_margin: f32,
    min_crafting_profit: i32,
    min_gathering_price: u32,
    min_gathering_velocity: f32,
    run_mode: RunMode,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let start = Instant::now();
    let library = Library::new()?;
    println!("Initialized in {} ms", start.elapsed().as_millis());

    let settings = Settings {
        listings_ratio: 1.1,
        min_profit_margin: 0.0,
        min_crafting_profit: 1000,
        min_crafting_velocity: 3.0,
        min_gathering_price: 1000,
        min_gathering_velocity: 3.0,
        // run_mode: RunMode::OnlyCustom,
        // run_mode: RunMode::OnlyCrafting,
        run_mode: RunMode::All,
    };

    // std::process::exit(0);
    let universalis = Universalis::get_mb_info(&library, &settings, "Siren", "Aether").await?;
    println!();

    library.write_files(&universalis, &settings)?;

    Ok(())
}
