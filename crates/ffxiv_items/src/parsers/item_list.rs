use anyhow::Result;
use csv::ReaderBuilder;
use std::io::Read;

use crate::ItemDBInfo;

#[derive(Default)]
pub struct ItemList(pub Vec<ItemDBInfo>);

impl ItemList {
    pub fn from_reader<R: Read>(reader: R) -> Result<Self> {
        let mut items = Vec::new();

        csv_parse!(reader => {
            id = U[0];
            name = S[9 + 1];
            ilevel = U[11 + 1];
            ui_category = U[15 + 1];
            equip_level = U[40 + 1];

            let item = ItemDBInfo {
                id,
                name: name.clone(),
                ui_category,
                ilevel,
                equip_level,
                ..Default::default()
            };

            items.push(item);
        });

        Ok(Self(items))
    }
}
