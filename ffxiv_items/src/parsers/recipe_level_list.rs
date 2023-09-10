use std::{collections::BTreeMap, path::Path};

use anyhow::Result;
use csv::ReaderBuilder;

pub struct RecipeLevelParsed {
    pub id: u32,
    pub level: u32,
    pub stars: u32,
}

#[derive(Default)]
pub struct RecipeLevelTable(pub BTreeMap<u32, RecipeLevelParsed>);

impl RecipeLevelTable {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut level_table = BTreeMap::new();

        csv_parse!(path => {
            id = U[0];
            level = U[1];
            stars = U[1 + 1];
            level_table.insert(id, RecipeLevelParsed { id, level, stars });
        });

        Ok(Self(level_table))
    }
}
