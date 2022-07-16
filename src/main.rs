mod cli;
mod library;
mod universalis;

use crate::{cli::process_cli, library::Library, universalis::Universalis};
pub use cli::{RunMode, Settings};
use std::{error::Error, time::Instant};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let settings = process_cli();

    let start = Instant::now();
    Library::download_files().await?;
    let library = Library::new()?;
    println!("Initialized in {} ms", start.elapsed().as_millis());

    let universalis = Universalis::get_mb_info(&library, &settings).await?;
    println!();

    library.write_files(&universalis, &settings)?;

    Ok(())
}
