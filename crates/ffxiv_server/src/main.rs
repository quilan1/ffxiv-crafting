#![warn(unused_crate_dependencies)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

mod market;
mod recipe;
mod responses;
mod server;

use std::{error::Error, time::Instant};

use responses::{JsonResponse, StringResponse};
use server::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    use ffxiv_items::{ItemDB, Library};

    setup()?;

    let library = Library::create().await?; // Safety: Initializing the singleton once

    let start = Instant::now();
    let item_db_conn = std::env::var("FFXIV_ITEM_DB_CONN").unwrap();
    let db = ItemDB::connect(item_db_conn).await?;
    db.initialize().await?;
    println!("Initialized in {} ms", start.elapsed().as_millis());

    Server::run(library, db).await?;

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
        log::info!(target: "ffxiv_server", "FFXIV_DATA_CENTERS is currently set to {val}");
    } else {
        log::info!(target: "ffxiv_server",
            "FFXIV_DATA_CENTERS environment variable not currently set. Defaulting to Dynamis."
        );
        std::env::set_var("FFXIV_DATA_CENTERS", "Dynamis");
    };

    if let Ok(val) = std::env::var("FFXIV_ITEM_DB_CONN") {
        log::info!(target: "ffxiv_server", "FFXIV_ITEM_DB_CONN is currently set to {val}");
    } else {
        let item_db = "user:password@localhost:3306";
        let msg = format!("FFXIV_ITEM_DB_CONN not set! Defaulting to {item_db}");
        println!("{msg}");
        log::warn!(target: "ffxiv_server", "{msg}");
        std::env::set_var("FFXIV_ITEM_DB_CONN", item_db);
    }

    if std::env::var("RUST_LIB_BACKTRACE").is_err() {
        std::env::set_var("RUST_LIB_BACKTRACE", "1");
    }

    let logfile = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%F %T%.3f)} | {({l}):5.5} | {({t}):17} | {m}{n}",
        )))
        .build("output.log")?;

    let config = Config::builder()
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .build(Root::builder().appender("logfile").build(LevelFilter::Info))?;

    log4rs::init_config(config)?;

    Ok(())
}

mod _temp {
    use axum_macros as _;
}
