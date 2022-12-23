#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

mod cli;
mod library;
mod server;
mod universalis;
mod util;

use std::{error::Error, time::Instant};

// #[tokio::main(flavor = "current_thread")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    use cli::process_cli;
    use library::Library;

    setup()?;
    process_cli();

    let start = Instant::now();
    Library::create().await?;
    println!("Initialized in {} ms", start.elapsed().as_millis());

    server::Server::run().await?;

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

    if std::env::var("RUST_LIB_BACKTRACE").is_err() {
        std::env::set_var("RUST_LIB_BACKTRACE", "1");
    }

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }

    console_subscriber::init();

    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{({d}):35} - {l} - {m}{n}")))
        .build("out/output.log")?;

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(LevelFilter::Info))?;

    log4rs::init_config(config)?;

    Ok(())
}
