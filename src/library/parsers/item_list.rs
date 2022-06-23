use anyhow::Result;
use csv::ReaderBuilder;
use std::{collections::BTreeMap, ops::Index, path::Path};

use crate::library::Library;

#[derive(Default)]
pub struct ItemList {
    pub name_to_id: BTreeMap<String, u32>,
    pub items: BTreeMap<u32, ItemInfo>,
}

pub struct ItemInfo {
    pub id: u32,
    pub name: String,
    pub ui_category: u32,
    pub ilevel: u32,
    pub equip_level: u32,
    pub is_untradable: bool,
}

impl ItemList {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut items = BTreeMap::new();
        let mut name_to_id = BTreeMap::new();

        let mut reader = ReaderBuilder::new().from_path(path)?;
        for (line, record) in reader.records().enumerate() {
            if line < 2 {
                continue;
            }

            let record = record?;
            let info = record.into_iter().collect::<Vec<_>>();
            let id = info[0].parse::<u32>()?;
            let name = info[9 + 1].to_string();
            let ilevel = info[11 + 1].parse::<u32>()?;
            let ui_category = info[15 + 1].parse::<u32>()?;
            let is_untradable = info[22 + 1] == "True";
            let equip_level = info[40 + 1].parse::<u32>()?;

            let item = ItemInfo {
                id,
                name: name.clone(),
                ui_category,
                ilevel,
                equip_level,
                is_untradable,
            };

            items.insert(id, item);
            name_to_id.insert(name, id);
        }

        Ok(Self { name_to_id, items })
    }

    pub fn all_craftable_items(&self, library: &Library) -> Vec<&ItemInfo> {
        self.items
            .iter()
            .filter_map(|(_, v)| {
                Some(v).filter(|item| library.all_recipes.contains_item_id(item.id))
            })
            .collect::<Vec<_>>()
    }

    pub fn all_gatherable_items(&self, library: &Library) -> Vec<&ItemInfo> {
        self.items
            .iter()
            .filter_map(|(_, v)| {
                Some(v).filter(|item| library.all_gathering.contains_item_id(&item.id))
            })
            .collect::<Vec<_>>()
    }
}

impl Index<&u32> for ItemList {
    type Output = ItemInfo;

    fn index(&self, index: &u32) -> &Self::Output {
        match self.items.get(index) {
            None => panic!("Missing item id: {index}"),
            Some(value) => &value,
        }
    }
}

impl Index<&str> for ItemList {
    type Output = u32;

    fn index(&self, index: &str) -> &Self::Output {
        match self.name_to_id.get(index) {
            None => panic!("Missing item name: {index}"),
            Some(value) => value,
        }
    }
}
