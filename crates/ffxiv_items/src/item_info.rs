use crate::{library, util::ItemId, Recipe};

#[derive(Default)]
pub struct ItemInfo {
    pub id: u32,
    pub name: String,
    pub ui_category: u32,
    pub ilevel: u32,
    pub equip_level: u32,
    pub is_untradable: bool,
    pub recipe: Option<Recipe>,
}

impl ItemInfo {
    pub fn get<I: ItemId>(obj: &I) -> &'static ItemInfo {
        let id = obj.item_id();
        &library().all_items[&id]
    }

    pub fn get_checked<I: ItemId>(obj: &I) -> Option<&'static ItemInfo> {
        library().all_items.items.get(&obj.item_id())
    }

    pub fn all_items() -> Vec<&'static ItemInfo> {
        library().all_items.items.values().collect::<Vec<_>>()
    }
}
