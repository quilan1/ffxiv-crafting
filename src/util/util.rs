#![allow(dead_code)]

pub use crate::library::library;
use crate::library::{Ingredient, ItemInfo};

////////////////////////////////////////////////////////////

pub trait ItemId {
    fn item_id(&self) -> u32;
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

impl ItemId for Ingredient {
    fn item_id(&self) -> u32 {
        self.item_id
    }
}

impl ItemId for &Ingredient {
    fn item_id(&self) -> u32 {
        self.item_id
    }
}

////////////////////////////////////////////////////////////

pub fn item_name<I: ItemId>(obj: &I) -> &'static str {
    let id = obj.item_id();
    &library().all_items[&id].name
}

pub fn item<I: ItemId>(obj: &I) -> &'static ItemInfo {
    let id = obj.item_id();
    &library().all_items[&id]
}

pub fn item_checked<I: ItemId>(obj: &I) -> Option<&'static ItemInfo> {
    let id = obj.item_id();
    library().all_items.items.get(&id)
}
