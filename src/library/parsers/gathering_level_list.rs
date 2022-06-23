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

        let mut reader = ReaderBuilder::new().from_path(path)?;
        for (line, record) in reader.records().enumerate() {
            if line < 2 {
                continue;
            }

            let record = record?;
            let info = record.into_iter().collect::<Vec<_>>();

            let id = info[0].parse::<u32>()?;
            let level = info[0 + 1].parse::<u32>()?;

            gathering_levels.insert(
                id,
                level,
            );
        }

        Ok(Self {
            gathering_levels,
        })
    }
}

impl Index<&u32> for GatheringLevelList {
    type Output = u32;

    fn index(&self, index: &u32) -> &Self::Output {
        match self.gathering_levels.get(index) {
            None => panic!("Missing gathering_level id: {index}"),
            Some(value) => &value,
        }
    }
}
