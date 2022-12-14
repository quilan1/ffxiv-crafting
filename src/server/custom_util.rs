use std::collections::{BTreeMap, BTreeSet};

use serde::Serialize;

use crate::{
    library::{library, Filter},
    universalis::{ItemListing, MarketItemInfo, MarketItemInfoMap},
    util::item_name,
};

use super::custom::CustomOut;

#[derive(Serialize, Debug)]
struct Recipe {
    inputs: Vec<RecipeData>,
    outputs: u32,
}

#[derive(Serialize, Debug)]
struct RecipeData {
    item_id: u32,
    count: u32,
}

#[derive(Serialize, Debug)]
pub struct CustomItemInfo {
    item_id: u32,
    name: String,
    listings: Vec<ItemListing>,
    history: Vec<ItemListing>,
    recipe: Option<Recipe>,
}

#[derive(Serialize, Debug)]
pub struct JsonFilter {
    name: String,
    values: Vec<String>,
}

pub fn get_ids_from_filters(filters: String) -> (Vec<u32>, Vec<u32>) {
    let item_list = library().all_items.items.values().collect::<Vec<_>>();
    let (items, _) = Filter::apply_filters(item_list, &filters);

    fn push_ids(ids: &mut Vec<u32>, item_id: u32) {
        ids.push(item_id);
        if !library().all_recipes.contains_item_id(item_id) {
            return;
        }

        for input in &library().all_recipes[&item_id].inputs {
            push_ids(ids, input.item_id);
        }
    }

    let ids = items
        .iter()
        .flat_map(|item| {
            let mut item_ids = Vec::new();
            push_ids(&mut item_ids, item.id);
            item_ids
        })
        // .filter(|item_id| !item(item_id).is_untradable)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    let items = items.into_iter().map(|item| item.id).collect::<Vec<_>>();

    (items, ids)
}

pub fn json_results(top_ids: Vec<u32>, mb_info: MarketItemInfoMap) -> CustomOut {
    let mut out_items = BTreeMap::new();
    for (id, MarketItemInfo { listings, history }) in mb_info {
        out_items.insert(
            id,
            CustomItemInfo {
                item_id: id,
                name: item_name(id).replace('\u{00A0}', " ").to_string(),
                listings,
                history,
                recipe: recipe_info(id),
            },
        );
    }

    for &id in &top_ids {
        if out_items.contains_key(&id) {
            continue;
        }

        out_items.insert(
            id,
            CustomItemInfo {
                item_id: id,
                name: item_name(id).to_string(),
                listings: Vec::new(),
                history: Vec::new(),
                recipe: recipe_info(id),
            },
        );
    }

    CustomOut {
        top_ids,
        item_info: out_items,
    }
}

fn recipe_info(id: u32) -> Option<Recipe> {
    library().all_recipes.get(&id).map(|recipe| Recipe {
        outputs: recipe.output.count,
        inputs: recipe
            .inputs
            .iter()
            .map(|input| RecipeData {
                item_id: input.item_id,
                count: input.count,
            })
            .collect(),
    })
}
