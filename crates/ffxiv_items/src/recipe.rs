use std::collections::BTreeMap;

pub trait AsIngredient {
    fn as_ingredient(&self) -> Ingredient;
}

#[derive(Clone, Default)]
pub struct Ingredient {
    pub count: u32,
    pub item_id: u32,
}

impl AsIngredient for u32 {
    fn as_ingredient(&self) -> Ingredient {
        Ingredient {
            item_id: *self,
            count: 1,
        }
    }
}

impl AsIngredient for &Ingredient {
    fn as_ingredient(&self) -> Ingredient {
        (*self).clone()
    }
}

#[derive(Clone)]
pub struct RecipeLevelInfo {
    pub level: u32,
    pub stars: u32,
}

#[derive(Clone)]
pub struct Recipe {
    pub output: Ingredient,
    pub inputs: Vec<Ingredient>,
    pub level: u32,
    pub stars: u32,
}

impl Recipe {
    pub fn to_map(values: Vec<Self>) -> BTreeMap<u32, Self> {
        values
            .into_iter()
            .map(|recipe| (recipe.output.item_id, recipe))
            .collect()
    }

    pub fn to_map_ref(values: &[Self]) -> BTreeMap<u32, &Self> {
        values
            .iter()
            .map(|recipe| (recipe.output.item_id, recipe))
            .collect()
    }

    pub fn to_vec(values: BTreeMap<u32, Self>) -> Vec<Self> {
        values.into_values().collect()
    }
}
