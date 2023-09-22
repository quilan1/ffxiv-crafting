use anyhow::Result;
use itertools::Itertools;
use std::{collections::BTreeMap, io::Read};

use crate::Ingredient;

pub struct RecipeParsed {
    pub output: Ingredient,
    pub inputs: Vec<Ingredient>,
    pub level_id: u32,
}

#[derive(Default)]
pub struct RecipeList(pub BTreeMap<u32, RecipeParsed>);

impl RecipeList {
    pub fn from_reader<R: Read>(reader: R) -> Result<Self> {
        let mut recipes = BTreeMap::new();

        csv_parse!(reader, info => {
            level_id = U[2 + 1];
            arr = U[4..24];

            let mut ingredients = Vec::new();
            for (item_id, count) in arr.into_iter().tuples() {
                if count > 0 {
                    ingredients.push(Ingredient { count, item_id });
                }
            }

            if ingredients.is_empty() {
                continue;
            }

            let (output, inputs) = ingredients.split_first().unwrap();
            let inputs = inputs.to_vec();
            recipes.insert(
                output.item_id,
                RecipeParsed {
                    output: output.clone(),
                    inputs: inputs.clone(),
                    level_id,
                },
            );
        });

        Ok(Self(recipes))
    }
}
