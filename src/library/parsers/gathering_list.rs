use anyhow::Result;
use csv::ReaderBuilder;
use std::{collections::BTreeMap, ops::Index, path::Path};

use crate::util::{item_checked, library};

#[derive(Default)]
pub struct GatheringList {
    by_item: BTreeMap<u32, Vec<u32>>,
    gathering: BTreeMap<u32, GatheringInfo>,
}

pub struct GatheringInfo {
    pub id: u32,
    pub item_id: u32,
    pub level: u32,
}

impl GatheringList {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut gathering = BTreeMap::new();
        let mut by_item = BTreeMap::<u32, Vec<u32>>::new();

        csv_parse!(path => {
            id = U[0];
            item_id = U[0 + 1];
            level = U[1 + 1];

            let level = library().all_gathering_levels[&level];
            match item_checked(item_id).map(|item| item.name.is_empty()) {
                None | Some(true) => continue,
                _ => {},
            };

            gathering.insert(id, GatheringInfo { id, item_id, level });

            by_item.entry(item_id).or_default().push(id);
        });

        Ok(Self {
            by_item,
            gathering,
        })
    }

    pub fn contains_item_id(&self, item_id: &u32) -> bool {
        self.by_item.contains_key(item_id)
    }
}

impl Index<&u32> for GatheringList {
    type Output = GatheringInfo;

    fn index(&self, index: &u32) -> &Self::Output {
        match self.gathering.get(index) {
            None => panic!("Missing gathering id: {index}"),
            Some(value) => value,
        }
    }
}
