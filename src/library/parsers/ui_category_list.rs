use std::{collections::BTreeMap, ops::Index, path::Path};

use anyhow::Result;
use csv::ReaderBuilder;

#[derive(Default)]
pub struct UiCategoryList {
    name_to_id: BTreeMap<String, u32>,
    categories: BTreeMap<u32, String>,
}

impl UiCategoryList {
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut categories = BTreeMap::new();
        let mut name_to_id = BTreeMap::new();

        csv_parse!(path => {
            id = U[0];
            name = S[1];
            categories.insert(id, name.to_string());
            name_to_id.insert(name.to_string(), id);
        });

        Ok(Self {
            categories,
            name_to_id,
        })
    }
}

impl Index<&String> for UiCategoryList {
    type Output = u32;

    fn index(&self, index: &String) -> &Self::Output {
        self.name_to_id.get(index).unwrap()
    }
}

impl Index<&u32> for UiCategoryList {
    type Output = String;

    fn index(&self, index: &u32) -> &Self::Output {
        self.categories.get(index).unwrap()
    }
}
