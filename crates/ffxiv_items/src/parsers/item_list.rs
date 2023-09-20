use anyhow::Result;
use csv::ReaderBuilder;
use std::{collections::BTreeMap, io::Read, ops::Index};

use crate::ItemInfo;

#[derive(Default)]
pub struct ItemList {
    pub name_to_id: BTreeMap<String, u32>,
    pub items: BTreeMap<u32, ItemInfo>,
}

impl ItemList {
    pub fn from_reader<R: Read>(reader: R) -> Result<Self> {
        let mut items = BTreeMap::new();
        let mut name_to_id = BTreeMap::new();

        csv_parse!(reader => {
            id = U[0];
            name = S[9 + 1];
            ilevel = U[11 + 1];
            ui_category = U[15 + 1];
            is_untradable = B[22 + 1];
            equip_level = U[40 + 1];

            let item = ItemInfo {
                id,
                name: name.clone(),
                ui_category,
                ilevel,
                equip_level,
                is_untradable,
                ..Default::default()
            };

            items.insert(id, item);
            name_to_id.insert(name, id);
        });

        Ok(Self { name_to_id, items })
    }
}

impl From<Vec<ItemInfo>> for ItemList {
    fn from(item_vec: Vec<ItemInfo>) -> Self {
        let mut items = BTreeMap::new();
        let mut name_to_id = BTreeMap::new();

        for item in item_vec {
            name_to_id.insert(item.name.clone(), item.id);
            items.insert(item.id, item);
        }

        Self { name_to_id, items }
    }
}

impl Index<&u32> for ItemList {
    type Output = ItemInfo;

    fn index(&self, index: &u32) -> &Self::Output {
        match self.items.get(index) {
            None => panic!("Missing item id: {index}"),
            Some(value) => value,
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
