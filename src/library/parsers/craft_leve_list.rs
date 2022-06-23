use anyhow::Result;
use csv::ReaderBuilder;
use std::{collections::BTreeMap, ops::Index, path::Path};

#[derive(Default)]
pub struct CraftLeveList {
    pub leves: BTreeMap<u32, u32>,
}

impl CraftLeveList {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut leves = BTreeMap::new();

        let mut reader = ReaderBuilder::new().from_path(path)?;
        for (line, record) in reader.records().enumerate() {
            if line < 2 {
                continue;
            }

            let record = record?;
            let info = record.into_iter().collect::<Vec<_>>();

            let id = info[0 + 1].parse::<u32>()?;
            let item = info[3 + 1].parse::<u32>()?;

            leves.insert(id, item);
        }

        Ok(Self { leves })
    }
}

impl Index<u32> for CraftLeveList {
    type Output = u32;

    fn index(&self, index: u32) -> &Self::Output {
        match self.leves.get(&index) {
            None => panic!("Missing leve id: {index}"),
            Some(value) => &value,
        }
    }
}
