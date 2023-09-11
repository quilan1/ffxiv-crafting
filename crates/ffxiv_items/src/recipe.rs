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

pub struct RecipeLevelInfo {
    pub level: u32,
    pub stars: u32,
}

pub struct Recipe {
    pub output: Ingredient,
    pub inputs: Vec<Ingredient>,
    pub level_info: Option<RecipeLevelInfo>,
}
