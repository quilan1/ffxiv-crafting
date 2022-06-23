use anyhow::Result;
use csv::ReaderBuilder;
use std::{collections::BTreeMap, ops::Index, path::Path};

use crate::Library;

#[derive(Default)]
pub struct LeveList {
    leves_by_item: BTreeMap<u32, Vec<u32>>,
    leves: BTreeMap<u32, LeveInfo>,
}

pub struct LeveInfo {
    pub id: u32,
    pub item: u32,
    pub level: u32,
    pub jobs: u32,
}

impl LeveList {
    pub fn from_path<P: AsRef<Path>>(path: P, library: &Library) -> Result<Self> {
        let mut leves = BTreeMap::new();
        let mut leves_by_item = BTreeMap::<u32, Vec<u32>>::new();

        let mut reader = ReaderBuilder::new().from_path(path)?;
        for (line, record) in reader.records().enumerate() {
            if line < 2 {
                continue;
            }

            let record = record?;
            let info = record.into_iter().collect::<Vec<_>>();

            let id = info[0].parse::<u32>()?;
            if !library.all_crafting_leves.leves.contains_key(&id) {
                continue;
            }

            let item = library.all_crafting_leves[id];
            let level = info[6 + 1].parse::<u32>()?;
            let jobs = info[15 + 1].parse::<u32>()?;

            leves.insert(
                id,
                LeveInfo {
                    id,
                    item,
                    level,
                    jobs,
                },
            );

            leves_by_item.entry(item).or_default().push(id);
        }

        Ok(Self {
            leves_by_item,
            leves,
        })
    }

    pub fn get_by_item_id(&self, index: &u32) -> Option<&Vec<u32>> {
        self.leves_by_item.get(index)
    }

    pub fn all_item_ids(&self) -> Vec<u32> {
        self.leves_by_item.keys().cloned().collect::<Vec<_>>()
    }
}

impl Index<&u32> for LeveList {
    type Output = LeveInfo;

    fn index(&self, index: &u32) -> &Self::Output {
        match self.leves.get(index) {
            None => panic!("Missing leve id: {index}"),
            Some(value) => &value,
        }
    }
}
