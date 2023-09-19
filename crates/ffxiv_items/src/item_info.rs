use itertools::Itertools;

use crate::{ItemId, Library, Recipe};

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
    pub fn all_recipe_input_ids<I: ItemId>(&self, library: &Library, item: I) -> Vec<u32> {
        fn inner(library: &Library, id: u32, results: &mut Vec<u32>) {
            let item = library.item_info(&id);
            results.push(item.id);

            if let Some(recipe) = &item.recipe {
                for input in &recipe.inputs {
                    inner(library, input.item_id(), results);
                }
            }
        }

        let mut results = Vec::new();
        if let Some(recipe) = &library.item_info(&item).recipe {
            for input in &recipe.inputs {
                inner(library, input.item_id(), &mut results);
            }
        }
        results.into_iter().unique().collect()
    }
}
