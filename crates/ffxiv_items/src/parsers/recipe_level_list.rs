use std::{collections::BTreeMap, io::Read};

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
    pub fn from_reader<R: Read>(reader: R) -> Result<Self> {
        let mut level_table = BTreeMap::new();

        csv_parse!(reader => {
            id = U[0];
            level = U[1];
            stars = U[1 + 1];
            level_table.insert(id, RecipeLevelParsed { id, level, stars });
        });

        Ok(Self(level_table))
    }
}
