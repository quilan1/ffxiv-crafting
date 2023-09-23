use std::collections::BTreeMap;

#[derive(Clone)]
pub struct Ingredient {
    pub count: u32,
    pub item_id: u32,
}

#[derive(Clone)]
pub struct Recipe {
    pub output: Ingredient,
    pub inputs: Vec<Ingredient>,
    pub level: u32,
    pub stars: u32,
}

impl Recipe {
    pub fn to_map_ref(values: &[Self]) -> BTreeMap<u32, &Self> {
        values
            .iter()
            .map(|recipe| (recipe.output.item_id, recipe))
            .collect()
    }
}
