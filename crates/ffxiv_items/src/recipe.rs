use std::collections::BTreeMap;

/// Holds information about inputs & outputs of a recipe.
#[derive(Clone, Debug)]
pub struct Ingredient {
    /// The number of items produced or consumed by a recipe.
    pub count: u32,
    /// The item_id of the item that is produced or consumed by a recipe.
    pub item_id: u32,
}

/// Crafting recipe information for items.
#[derive(Clone, Debug)]
pub struct Recipe {
    /// An `Ingredient` representing the output item of a recipe.
    pub output: Ingredient,
    /// The various `Ingredients` that go into crafting a recipe.
    pub inputs: Vec<Ingredient>,
    /// The character level required to craft this recipe.
    pub level: u32,
    pub(crate) stars: u32,
}

impl Recipe {
    pub(crate) fn to_map_ref(values: &[Self]) -> BTreeMap<u32, &Self> {
        values
            .iter()
            .map(|recipe| (recipe.output.item_id, recipe))
            .collect()
    }
}
