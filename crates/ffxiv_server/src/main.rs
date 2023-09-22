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

////////////////////////////////////////////////////////////

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    use ffxiv_items::ItemDB;

    setup();

    let start = Instant::now();
    let item_db_conn = std::env::var("FFXIV_ITEM_DB_CONN").unwrap();
    let db = ItemDB::connect(item_db_conn).await?;
    db.initialize().await?;
    println!("Initialized in {} ms", start.elapsed().as_millis());

    Server::run(db).await?;

    Ok(())
}

////////////////////////////////////////////////////////////

fn setup() {
    use chrono::Local;
    use log::LevelFilter;
    use std::io::Write;

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

    let file_target = Box::new(FileAppender::new("output.log"));
    env_logger::Builder::new()
        .target(env_logger::Target::Pipe(file_target))
        .filter(None, LevelFilter::Info)
        .filter(Some("sqlx"), LevelFilter::Error)
        .format(|buf, record| {
            writeln!(
                buf,
                "{} | {:5} | {:17} | {}",
                Local::now().format("%F %T%.3f"),
                record.level(),
                record.target(),
                record.args()
            )
        })
        .init();
}

////////////////////////////////////////////////////////////
struct FileAppender {
    file_name: String,
}

impl FileAppender {
    fn new<S: Into<String>>(file_name: S) -> Self {
        Self {
            file_name: file_name.into(),
        }
    }
}

impl std::io::Write for FileAppender {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut data_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.file_name)?;

        data_file.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

////////////////////////////////////////////////////////////

mod _temp {
    use axum_macros as _;
}
