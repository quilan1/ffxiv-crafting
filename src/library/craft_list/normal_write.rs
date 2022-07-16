use anyhow::Result;
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use super::{AnalysisFilters, CraftList};
use crate::{
    library::{Library, RecursiveMarketBoardAnalysis},
    universalis::Universalis,
    Settings,
};

impl CraftList {
    pub fn write_to_file<P: AsRef<Path>>(
        &self,
        path: P,
        library: &Library,
        universalis: &Universalis,
        settings: &Settings,
    ) -> Result<()> {
        let writer = &mut BufWriter::new(File::create(path.as_ref())?);

        for group in &self.craft_groups {
            write!(writer, "== {} ==\n", group.heading)?;

            let analysis_filters = AnalysisFilters::from_filters(&group.filters)?;
            let analyses = group
                .crafts
                .iter()
                .filter_map(|craft| {
                    match RecursiveMarketBoardAnalysis::analyze(
                        craft.item_id,
                        library,
                        universalis,
                        settings,
                        1,
                        true,
                        &analysis_filters,
                    ) {
                        None => None,
                        Some(mut analysis) => {
                            analysis.analysis.profit = analysis.analysis.sell_price as i32
                                - analysis.best_buy_price as i32;
                            Some(analysis)
                        }
                    }
                })
                .collect::<Vec<_>>();

            let min_crafting_velocity = analysis_filters
                .min_velocity
                .unwrap_or(settings.min_crafting_velocity);

            let mut analyses = analyses
                .into_iter()
                .filter(|analysis| {
                    analysis.analysis.profit >= settings.min_crafting_profit
                        && analysis.analysis.velocity_info_nq.velocity
                            + analysis.analysis.velocity_info_hq.velocity
                            >= min_crafting_velocity
                })
                .collect::<Vec<_>>();

            analyses.sort_by_key(|analysis| analysis.analysis.profit);
            if let Some(limit) = analysis_filters.limit {
                analyses.reverse();
                analyses.truncate(limit as usize);
                analyses.reverse();
            }

            for analysis in analyses {
                write!(
                    writer,
                    "{:<40}| {:<24}| {:<24}| {:<8}{:<8}{:<8}\n",
                    library.all_items[&analysis.analysis.item_id].name,
                    analysis.analysis.velocity_info_nq.to_string(),
                    analysis.analysis.velocity_info_hq.to_string(),
                    analysis.analysis.sell_price,
                    analysis.best_buy_price,
                    analysis.analysis.profit,
                )?;
            }
            write!(writer, "\n")?;
        }

        Ok(())
    }
}
