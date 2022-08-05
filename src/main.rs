mod cli;
mod library;
mod server;
mod universalis;
mod util;

use std::{error::Error, time::Instant};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    use cli::process_cli;
    use library::{library, JsWriter, Library};
    use universalis::Universalis;

    process_cli();

    let start = Instant::now();
    Library::create().await?;
    println!("Initialized in {} ms", start.elapsed().as_millis());

    let start = Instant::now();
    let universalis = Universalis::get_mb_info().await?;
    println!("Pulled mb data in {} ms", start.elapsed().as_millis());
    println!();

    let start = Instant::now();
    library().write_files(&universalis)?;
    JsWriter::write_all(&universalis)?;
    println!("Wrote data in {} ms", start.elapsed().as_millis());

    server::Server::run().await?;

    Ok(())
}
