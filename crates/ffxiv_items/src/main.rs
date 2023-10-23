use std::time::Instant;

use anyhow::Result;
use mock_traits::ReqwestDownloader;

#[tokio::main]
async fn main() -> Result<()> {
    use ffxiv_items::ItemDB;
    println!("Checking database status");

    setup();

    let start = Instant::now();
    let item_db_conn = std::env::var("FFXIV_ITEM_DB_CONN").unwrap();
    let db = ItemDB::connect(item_db_conn).await?;
    if db.initialize::<ReqwestDownloader>().await? {
        println!("Initialized in {} ms", start.elapsed().as_millis());
    } else {
        println!("Done!");
    }
    Ok(())
}

fn setup() {
    use chrono::Local;
    use std::io::Write;

    if let Ok(val) = std::env::var("FFXIV_ITEM_DB_CONN") {
        log::info!(target: "ffxiv_server", "FFXIV_ITEM_DB_CONN is currently set to {val}");
    } else {
        let item_db = "mysql://user:password@localhost:3306/ffxiv_items";
        let msg = format!("FFXIV_ITEM_DB_CONN not set! Defaulting to {item_db}");
        println!("{msg}");
        log::warn!(target: "ffxiv_server", "{msg}");
        std::env::set_var("FFXIV_ITEM_DB_CONN", item_db);
    }

    env_logger::Builder::new()
        .filter(None, log::LevelFilter::Warn)
        .filter(Some("sqlx"), log::LevelFilter::Error)
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
