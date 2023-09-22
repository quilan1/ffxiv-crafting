use crate::{Ingredient, ItemInfo};

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

impl ItemId for ItemInfo {
    fn item_id(&self) -> u32 {
        self.id
    }
}

impl ItemId for &ItemInfo {
    fn item_id(&self) -> u32 {
        self.id
    }
}
