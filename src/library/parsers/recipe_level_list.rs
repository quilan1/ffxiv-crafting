use std::{collections::BTreeMap, ops::Index, path::Path};

use anyhow::Result;
use csv::ReaderBuilder;

pub struct RecipeLevel {
    pub id: u32,
    pub level: u32,
    pub stars: u32,
}

#[derive(Default)]
pub struct RecipeLevelTable {
    level_table: BTreeMap<u32, RecipeLevel>,
}

impl RecipeLevelTable {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut level_table = BTreeMap::new();

        let mut reader = ReaderBuilder::new().from_path(path)?;
        for (line, record) in reader.records().enumerate() {
            let record = record?;
            if line < 2 {
                continue;
            }

            let id = record[0].parse::<u32>()?;
            let level = record[0 + 1].parse::<u32>()?;
            let stars = record[1 + 1].parse::<u32>()?;
            level_table.insert(id, RecipeLevel { id, level, stars });
        }

        Ok(Self { level_table })
    }
}

impl Index<&u32> for RecipeLevelTable {
    type Output = RecipeLevel;

    fn index(&self, index: &u32) -> &Self::Output {
        // println!("Looking for RecipeLevelTable {}", index);
        self.level_table.get(index).unwrap()
    }
}
