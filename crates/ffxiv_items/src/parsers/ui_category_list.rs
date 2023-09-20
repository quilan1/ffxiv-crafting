use std::{collections::BTreeMap, io::Read, ops::Index};

use anyhow::Result;
use csv::ReaderBuilder;

#[derive(Default)]
pub struct UiCategoryList {
    name_to_id: BTreeMap<String, u32>,
    categories: BTreeMap<u32, String>,
}

impl UiCategoryList {
    pub fn from_reader<R: Read>(reader: R) -> Result<Self> {
        let mut categories = BTreeMap::new();
        let mut name_to_id = BTreeMap::new();

        csv_parse!(reader => {
            id = U[0];
            name = S[1];
            categories.insert(id, name.to_string());
            name_to_id.insert(name.to_string(), id);
        });

        Ok(Self {
            name_to_id,
            categories,
        })
    }
}

impl From<&[&str]> for UiCategoryList {
    fn from(categories_vec: &[&str]) -> Self {
        let mut categories = BTreeMap::new();
        let mut name_to_id = BTreeMap::new();

        for (id, cat) in categories_vec.iter().enumerate() {
            let id = id as u32 + 1;
            name_to_id.insert(cat.to_string(), id);
            categories.insert(id, cat.to_string());
        }

        Self {
            name_to_id,
            categories,
        }
    }
}

impl Index<&String> for UiCategoryList {
    type Output = u32;

    fn index(&self, index: &String) -> &Self::Output {
        self.name_to_id.get(index).unwrap()
    }
}

impl Index<&str> for UiCategoryList {
    type Output = u32;

    fn index(&self, index: &str) -> &Self::Output {
        self.name_to_id.get(index).unwrap()
    }
}

impl Index<&u32> for UiCategoryList {
    type Output = String;

    fn index(&self, index: &u32) -> &Self::Output {
        self.categories.get(index).unwrap()
    }
}
