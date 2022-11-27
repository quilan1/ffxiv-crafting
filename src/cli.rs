use clap::{arg, command, value_parser};

#[derive(PartialEq)]
pub enum RunMode {
    OnlyCustom,
    OnlyGathering,
    OnlyCrafting,
    All,
}

#[allow(dead_code)]
pub struct Settings {
    pub listings_ratio: f32,
    pub max_crafting_allowed_recursive_loss: u32,
    pub min_crafting_profit_margin: f32,
    pub min_crafting_velocity: f32,
    pub min_crafting_profit: i32,
    pub min_gathering_price: u32,
    pub min_gathering_velocity: f32,
    pub history_length: f32,
    pub homeworld: String,
    pub data_centers: Vec<String>,
    pub run_mode: RunMode,
    pub characters: Vec<String>,
}

static mut SETTINGS: Option<Settings> = None;

pub fn process_cli() {
    let matches = command!()
        .arg(arg!(--homeworld [VALUE] "Homeworld server"))
        .arg(arg!(--data_centers [VALUES] "Data Centers, comma-separated"))
        .arg(
            arg!(--min_profit [VALUE] "Minimum crafting profit value")
                .value_parser(value_parser!(i32)),
        )
        .arg(
            arg!(--min_velocity [VALUE] "Minimum crafting velocity")
                .value_parser(value_parser!(f32)),
        )
        .arg(arg!(--only_gathering "Only gathering items"))
        .arg(arg!(--only_crafting "Only crafting items"))
        .arg(arg!(--only_custom "Only custom items"))
        .get_matches();

    let mut settings = Settings {
        listings_ratio: 1.1,
        max_crafting_allowed_recursive_loss: 1000,
        min_crafting_profit_margin: 0.0,
        min_crafting_profit: 1000,
        min_crafting_velocity: 3.0,
        min_gathering_price: 400,
        min_gathering_velocity: 3.0,
        history_length: 2.0,
        run_mode: RunMode::All,
        homeworld: "Seraph".into(),
        data_centers: vec!["Dynamis".into()],
        characters: vec!["Quilan", "Vernox", "Veronixia"]
            .into_iter()
            .map(|c| c.into())
            .collect::<Vec<_>>(),
    };

    if let Some(value) = matches.get_one::<String>("homeworld") {
        settings.homeworld = value.clone();
    }

    if let Some(value) = matches.get_one::<String>("data_centers") {
        settings.data_centers = value.split(",").map(|v| v.trim().into()).collect();
    }

    if let Some(&value) = matches.get_one::<i32>("min_profit") {
        settings.min_crafting_profit = value;
    }

    if let Some(&value) = matches.get_one::<f32>("min_velocity") {
        settings.min_crafting_velocity = value;
    }

    if matches.contains_id("only_gathering") {
        settings.run_mode = RunMode::OnlyGathering;
    }

    if matches.contains_id("only_crafting") {
        settings.run_mode = RunMode::OnlyCrafting;
    }

    if matches.contains_id("only_custom") {
        settings.run_mode = RunMode::OnlyCustom;
    }

    unsafe {
        SETTINGS = Some(settings);
    }
}

pub fn settings() -> &'static Settings {
    unsafe { SETTINGS.as_ref().expect("SETTINGS value not yet set!") }
}
