use anyhow::Result;
use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use super::{AnalysisFilters, CraftList};
use crate::{
    library::{Ingredient, Library, RecursiveMarketBoardAnalysis},
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

            let mut analyses = Vec::new();
            for recipe in &group.crafts {
                let analysis_filters = AnalysisFilters::new(&group.filters, &recipe.filters)?;
                let multiplier = analysis_filters.count.unwrap_or(1);

                let rec_analysis = match RecursiveMarketBoardAnalysis::analyze(
                    recipe.item_id,
                    library,
                    universalis,
                    settings,
                    multiplier,
                    true,
                    &analysis_filters,
                ) {
                    None => continue,
                    Some(v) => v,
                };

                purchases.extend(rec_analysis.all_purchases());
                analyses.push((rec_analysis, analysis_filters));
            }

            analyses.sort_by_key(|(analysis, _)| analysis.best_buy_price);

            for (index, (analysis, _filters)) in analyses.into_iter().enumerate() {
                if let Some(limit) = _filters.limit {
                    if index >= limit as usize {
                        break;
                    }
                }
                analysis.write(&mut writer, library)?;
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
                    &AnalysisFilters::default(),
                )
            })
            .collect::<Vec<_>>();

        for analysis in analyses {
            analysis.write(&mut writer, library)?;
        }

        Ok(())
    }
}

impl RecursiveMarketBoardAnalysis {
    fn write<W: Write>(&self, writer: &mut W, library: &Library) -> Result<()> {
        self.write_depth(writer, library, "".into())
    }

    fn write_depth<W: Write>(
        &self,
        writer: &mut W,
        library: &Library,
        indent: String,
    ) -> Result<()> {
        let item = &library.all_items[&self.ingredient.item_id];
        let name = format!(
            "{indent}{}x {} ({:.1})",
            self.ingredient.count,
            item.name,
            self.analysis.velocity_info_hq.velocity + self.analysis.velocity_info_nq.velocity
        );

        if !self.children.is_empty() {
            let new_indent = format!("{indent}  ");
            write!(
                writer,
                "{name:<40}| {:<8}{:<8}| {}\n",
                self.analysis.buy_price, self.best_buy_price, ""
            )?;
            for child in &self.children {
                child.write_depth(writer, library, new_indent.clone())?;
            }
        } else {
            write!(
                writer,
                "{name:<40}| {:<8}{:<8}| {:?}\n",
                self.best_buy_price, self.best_buy_price, self.analysis.buy_worlds
            )?;
        }

        Ok(())
    }
}
