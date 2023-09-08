use itertools::Itertools;
use ffxiv_items::{library, Filter};

pub fn get_ids_from_filters<S: AsRef<str>>(filters: S) -> (Vec<u32>, Vec<u32>) {
    fn push_ids(ids: &mut Vec<u32>, item_id: u32) {
        ids.push(item_id);
        if !library().all_recipes.contains_item_id(item_id) {
            return;
        }

        for input in &library().all_recipes[&item_id].inputs {
            push_ids(ids, input.item_id);
        }
    }

    let filters = filters.as_ref();
    let item_list = library().all_items.items.values().collect::<Vec<_>>();
    let (items, _) = Filter::apply_filters(item_list, filters);

    let item_ids = items.into_iter().map(|item| item.id).collect::<Vec<_>>();

    let ids = item_ids
        .iter()
        .flat_map(|&id| {
            let mut item_ids = Vec::new();
            push_ids(&mut item_ids, id);
            item_ids
        })
        .unique()
        .collect::<Vec<_>>();

    (item_ids, ids)
}
