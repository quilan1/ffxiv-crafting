use anyhow::Result;
use csv::ReaderBuilder;
use std::{collections::BTreeMap, ops::Index, path::Path};

#[derive(Default)]
pub struct GatheringLevelList {
    gathering_levels: BTreeMap<u32, u32>,
}

impl GatheringLevelList {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut gathering_levels = BTreeMap::new();

        csv_parse!(path => {
            id = U[0];
            level = U[1];
            gathering_levels.insert(id, level);
        });

        Ok(Self { gathering_levels })
    }
}

impl Index<&u32> for GatheringLevelList {
    type Output = u32;

    fn index(&self, index: &u32) -> &Self::Output {
        match self.gathering_levels.get(index) {
            None => panic!("Missing gathering_level id: {index}"),
            Some(value) => value,
        }
    }
}
