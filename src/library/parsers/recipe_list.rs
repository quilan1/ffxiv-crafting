use anyhow::Result;
use csv::ReaderBuilder;
use itertools::Itertools;
use std::{collections::BTreeMap, ops::Index, path::Path};

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

pub struct Recipe {
    pub output: Ingredient,
    pub inputs: Vec<Ingredient>,
    pub level_id: u32,
}

#[derive(Default)]
pub struct RecipeList {
    pub recipes: BTreeMap<u32, Recipe>,
}

impl RecipeList {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut recipes = BTreeMap::new();

        csv_parse!(path => {
            level_id = U[2 + 1];
            arr = U[4..24];

            let mut ingredients = Vec::new();
            for (item_id, count) in arr.into_iter().tuples() {
                if count > 0 {
                    ingredients.push(Ingredient { item_id, count });
                }
            }

            if ingredients.len() == 0 {
                continue;
            }

            let (output, inputs) = ingredients.split_first().unwrap();
            let inputs = inputs.to_vec();
            recipes.insert(
                output.item_id,
                Recipe {
                    output: output.clone(),
                    inputs: inputs.to_vec(),
                    level_id,
                },
            );
        });

        Ok(Self { recipes })
    }

    pub fn contains_item_id(&self, id: u32) -> bool {
        self.recipes.contains_key(&id)
    }

    pub fn get(&self, id: &u32) -> Option<&Recipe> {
        self.recipes.get(id)
    }
}

impl Index<&u32> for RecipeList {
    type Output = Recipe;

    fn index(&self, index: &u32) -> &Self::Output {
        match self.recipes.get(index) {
            None => panic!("Missing item id: {index}"),
            Some(value) => value,
        }
    }
}
