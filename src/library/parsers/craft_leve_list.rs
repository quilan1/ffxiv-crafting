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

        csv_parse!(path => {
            id = U[0 + 1];
            item = U[3 + 1];
            leves.insert(id, item);
        });

        Ok(Self { leves })
    }
}

impl Index<u32> for CraftLeveList {
    type Output = u32;

    fn index(&self, index: u32) -> &Self::Output {
        match self.leves.get(&index) {
            None => panic!("Missing leve id: {index}"),
            Some(value) => value,
        }
    }
}
