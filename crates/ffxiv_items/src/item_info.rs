use itertools::Itertools;

use crate::{library, util::ItemId, Recipe};

#[derive(Clone, Default)]
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

    pub fn all_recipe_input_ids<I: ItemId>(&self, item: I) -> Vec<u32> {
        fn inner(id: u32, results: &mut Vec<u32>) {
            let item = ItemInfo::get(&id);
            results.push(item.id);

            if let Some(recipe) = &item.recipe {
                for input in &recipe.inputs {
                    inner(input.item_id(), results);
                }
            }
        }

        let mut results = Vec::new();
        if let Some(recipe) = &ItemInfo::get(&item).recipe {
            for input in &recipe.inputs {
                inner(input.item_id(), &mut results);
            }
        }
        results.into_iter().unique().collect()
    }
}
