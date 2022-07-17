mod cli;
mod library;
mod universalis;
mod util;

use std::{error::Error, time::Instant};

pub use cli::{RunMode, Settings};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    use cli::process_cli;
    use library::{library, Library};
    use universalis::Universalis;

    let settings = process_cli();

    let start = Instant::now();
    Library::create().await?;
    println!("Initialized in {} ms", start.elapsed().as_millis());

    let start = Instant::now();
    let universalis = Universalis::get_mb_info(&settings).await?;
    println!("Pulled mb data in {} ms", start.elapsed().as_millis());
    println!();

    let start = Instant::now();
    library().write_files(&universalis, &settings)?;
    println!("Wrote data in {} ms", start.elapsed().as_millis());

    Ok(())
}
