use anyhow::Result;
use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use super::CraftList;
use crate::{
    library::{Ingredient, Library},
    market_board_analysis::RecursiveMarketBoardAnalysis,
    universalis::Universalis,
    Settings,
};

impl CraftList {
    pub fn write_custom_to_file<P: AsRef<Path>>(
        &self,
        path: P,
        library: &Library,
        universalis: &Universalis,
        settings: &Settings,
    ) -> Result<()> {
        let mut writer = BufWriter::new(File::create(path.as_ref())?);

        let mut purchases = Vec::new();
        for group in &self.craft_groups {
            write!(&mut writer, "== {} ==\n", group.heading)?;

            for recipe in &group.crafts {
                let multiplier = recipe
                    .filters
                    .iter()
                    .find(|filter| filter.ftype == ":count")
                    .map(|filter| {
                        filter
                            .options
                            .first()
                            .map(|value| value.parse::<u32>().unwrap_or(1))
                    })
                    .flatten()
                    .unwrap_or(1);
                let rec_analysis = match RecursiveMarketBoardAnalysis::analyze(
                    recipe.item_id,
                    library,
                    universalis,
                    settings,
                    multiplier,
                    true,
                ) {
                    None => continue,
                    Some(v) => v,
                };

                purchases.extend(rec_analysis.all_purchases());

                rec_analysis.write(&mut writer, library)?;
                write!(&mut writer, "\n")?;
            }
        }

        write!(&mut writer, "\n=== ALL ITEMS ===\n")?;

        let mut purchase_items = BTreeMap::<u32, u32>::new();
        purchases.into_iter().for_each(|ingredient| {
            let entry = purchase_items.entry(ingredient.item_id).or_default();
            *entry += ingredient.count;
        });

        let analyses = purchase_items
            .into_iter()
            .map(|(item_id, count)| Ingredient { count, item_id })
            .filter_map(|ingredient| {
                RecursiveMarketBoardAnalysis::analyze(
                    &ingredient,
                    library,
                    universalis,
                    settings,
                    1,
                    false,
                )
            })
            .collect::<Vec<_>>();

        for analysis in analyses {
            analysis.write(&mut writer, library)?;
        }

        Ok(())
    }
}
