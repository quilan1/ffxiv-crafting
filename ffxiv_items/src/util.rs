#![allow(dead_code)]

use itertools::Itertools;

use crate::{library, Filter, Ingredient, ItemInfo};

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

////////////////////////////////////////////////////////////

pub fn get_ids_from_filters<S: AsRef<str>>(filters: S) -> (Vec<u32>, Vec<u32>) {
    fn push_ids(ids: &mut Vec<u32>, item_id: u32) {
        ids.push(item_id);

        if let Some(recipe) = library()
            .all_items
            .items
            .get(&item_id)
            .and_then(|item| item.recipe.as_ref())
        {
            for input in &recipe.inputs {
                push_ids(ids, input.item_id);
            }
        }
    }

    fn inner(filters: &str) -> (Vec<u32>, Vec<u32>) {
        let (items, _) = Filter::apply_filters(ItemInfo::all_items(), filters);
        let top_level_item_ids = items.into_iter().map(|item| item.id).collect::<Vec<_>>();
        let all_item_ids = top_level_item_ids
            .iter()
            .flat_map(|&id| {
                let mut item_ids = Vec::new();
                push_ids(&mut item_ids, id);
                item_ids
            })
            .unique()
            .collect::<Vec<_>>();

        (top_level_item_ids, all_item_ids)
    }

    inner(filters.as_ref())
}
