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
        let mut reader = ReaderBuilder::new().from_path(path)?;
        for (line, record) in reader.records().enumerate() {
            let record = record?;
            if line < 2 {
                continue;
            }

            if record.len() < 18 {
                continue;
            }

            let level_id = record[2 + 1].parse::<u32>().unwrap();
            let arr = (4..18).map(|ind| &record[ind]).collect::<Vec<_>>();

            let mut ingredients = Vec::new();
            for (id, count) in arr.into_iter().tuples() {
                let id = id.parse::<u32>()?;
                let count = count.parse::<u32>()?;
                if count > 0 {
                    ingredients.push((id, count));
                }
            }

            if ingredients.len() == 0 {
                continue;
            }

            let (id, count) = ingredients[0];
            recipes.insert(
                id,
                Recipe {
                    output: Ingredient { item_id: id, count },
                    inputs: ingredients[1..]
                        .into_iter()
                        .map(|&(id, count)| Ingredient { item_id: id, count })
                        .collect::<Vec<_>>(),
                    level_id,
                },
            );
        }

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
