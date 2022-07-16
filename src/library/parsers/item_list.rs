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

        csv_parse!(path => {
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
            };

            items.insert(id, item);
            name_to_id.insert(name, id);
        });

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

///////////////////////////////////////////

pub trait ItemId {
    fn item_id(&self) -> u32;
}

static mut ITEM_LIST: Option<ItemList> = None;

pub fn item_list() -> &'static ItemList {
    unsafe { ITEM_LIST.as_ref().expect("ITEM_LIST has not been set!") }
}

pub fn item_name<I: ItemId>(obj: I) -> &'static str {
    let id = obj.item_id();
    &item_list()[&id].name
}

pub fn item<I: ItemId>(obj: I) -> &'static ItemInfo {
    let id = obj.item_id();
    &item_list()[&id]
}

impl ItemId for u32 {
    fn item_id(&self) -> u32 {
        *self
    }
}

impl ItemId for &u32 {
    fn item_id(&self) -> u32 {
        **self
    }
}
