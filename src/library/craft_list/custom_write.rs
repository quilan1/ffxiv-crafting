use anyhow::Result;
use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use super::{AnalysisFilters, CraftList};
use crate::{
    library::{market_board_analysis::WorldInfo, Ingredient, RecursiveMarketBoardAnalysis},
    universalis::Universalis,
    util::{item, item_name},
};

impl CraftList {
    pub fn write_custom_to_file<P: AsRef<Path>>(
        &self,
        path: P,
        universalis: &Universalis,
    ) -> Result<()> {
        let writer = &mut BufWriter::new(File::create(path.as_ref())?);

        for group in &self.craft_groups {
            write!(writer, "== {} ==\n", group.heading)?;

            let mut purchases = Vec::new();
            let mut analyses = Vec::new();
            for recipe in &group.crafts {
                let analysis_filters = AnalysisFilters::new(&group.filters, &recipe.filters)?;
                let multiplier = analysis_filters.count.unwrap_or(1);

                let rec_analysis = match RecursiveMarketBoardAnalysis::analyze(
                    recipe.item_id,
                    universalis,
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
                analysis.write(writer)?;
                write!(writer, "\n")?;
            }

            write!(writer, "=== ALL ITEMS ===\n")?;
            write_sorted_analyses(writer, purchases, universalis)?;
            write!(writer, "\n")?;
        }

        Ok(())
    }
}

fn write_sorted_analyses<W: Write>(
    writer: &mut W,
    purchases: Vec<Ingredient>,
    universalis: &Universalis,
) -> Result<()> {
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
                universalis,
                1,
                false,
                &AnalysisFilters::default(),
            )
        })
        .collect::<Vec<_>>();

    struct Info<'a> {
        item_id: u32,
        world_info: &'a WorldInfo,
    }

    let mut world_items = BTreeMap::<String, Vec<Info>>::new();

    for analysis in &analyses {
        for world_info in &analysis.analysis.buy_worlds {
            let entry = world_items.entry(world_info.world.clone()).or_default();
            entry.push(Info {
                item_id: analysis.ingredient.item_id,
                world_info: &world_info,
            });
        }
    }

    for (world, items) in world_items {
        write!(writer, "{world}:\n")?;
        for Info {
            item_id,
            world_info: WorldInfo { count, price, .. },
        } in items
        {
            write!(writer, "\t{:40}| {count:8} <{price}\n", item_name(item_id))?;
        }
    }

    Ok(())
}

impl RecursiveMarketBoardAnalysis {
    fn write<W: Write>(&self, writer: &mut W) -> Result<()> {
        self.write_depth(writer, "".into())
    }

    fn write_depth<W: Write>(&self, writer: &mut W, indent: String) -> Result<()> {
        let item = item(&self.ingredient);
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
                child.write_depth(writer, new_indent.clone())?;
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
