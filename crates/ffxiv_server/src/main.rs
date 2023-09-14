#![warn(unused_crate_dependencies)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

mod gen_listing;
mod recipe;
mod server;
mod util;

use std::{error::Error, time::Instant};

use crate::gen_listing::ListingInfo;
use crate::server::Server;

// #[tokio::main(flavor = "current_thread")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    use ffxiv_items::Library;

    setup()?;

    let start = Instant::now();
    unsafe { Library::create().await? }; // Safety: Initializing the singleton once
    println!("Initialized in {} ms", start.elapsed().as_millis());

    Server::run().await?;

    Ok(())
}

fn setup() -> Result<(), Box<dyn Error>> {
    use log::LevelFilter;
    use log4rs::{
        append::file::FileAppender,
        config::{Appender, Root},
        encode::pattern::PatternEncoder,
        Config,
    };

    if let Ok(val) = std::env::var("FFXIV_DATA_CENTERS") {
        println!("FFXIV_DATA_CENTERS is currently set to {val}");
    } else {
        println!(
            "FFXIV_DATA_CENTERS environment variable not currently set. Defaulting to Dynamis."
        );
        std::env::set_var("FFXIV_DATA_CENTERS", "Dynamis");
    };

    if std::env::var("RUST_LIB_BACKTRACE").is_err() {
        std::env::set_var("RUST_LIB_BACKTRACE", "1");
    }

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{({d}):35} - {l} - {m}{n}")))
        .build("out/output.log")?;

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(LevelFilter::Info))?;

    log4rs::init_config(config)?;

    Ok(())
}

#[cfg(test)]
mod tests {}
