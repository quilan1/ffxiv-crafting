use itertools::Itertools;

use crate::{Filter, Ingredient, ItemInfo, Library};

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

////////////////////////////////////////////////////////////

pub fn get_ids_from_filters<S: AsRef<str>>(library: &Library, filters: S) -> (Vec<u32>, Vec<u32>) {
    fn push_ids(library: &Library, ids: &mut Vec<u32>, item_id: u32) {
        ids.push(item_id);

        if let Some(recipe) = library
            .all_items
            .items
            .get(&item_id)
            .and_then(|item| item.recipe.as_ref())
        {
            for input in &recipe.inputs {
                push_ids(library, ids, input.item_id);
            }
        }
    }

    fn inner(library: &Library, filters: &str) -> (Vec<u32>, Vec<u32>) {
        let items = Filter::apply_filter_str(library, filters, library.all_items());
        let top_level_item_ids = items.into_iter().map(|item| item.id).collect::<Vec<_>>();
        let all_item_ids = top_level_item_ids
            .iter()
            .flat_map(|&id| {
                let mut item_ids = Vec::new();
                push_ids(library, &mut item_ids, id);
                item_ids
            })
            .unique()
            .collect::<Vec<_>>();

        (top_level_item_ids, all_item_ids)
    }

    inner(library, filters.as_ref())
}
