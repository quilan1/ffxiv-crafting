use anyhow::Result;
use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use super::CraftList;
use crate::{
    library::Library, market_board_analysis::RecursiveMarketBoardAnalysis,
    universalis::Universalis, Settings,
};

impl CraftList {
    pub fn write_to_file<P: AsRef<Path>>(
        &self,
        path: P,
        library: &Library,
        universalis: &Universalis,
        settings: &Settings,
    ) -> Result<()> {
        let mut writer = BufWriter::new(File::create(path.as_ref())?);

        for group in &self.craft_groups {
            write!(&mut writer, "== {} ==\n", group.heading)?;

            let mut analyses = group
                .crafts
                .iter()
                .filter_map(|craft| {
                    RecursiveMarketBoardAnalysis::analyze(
                        craft.item_id,
                        library,
                        universalis,
                        settings,
                        1,
                        true,
                    )
                })
                .collect::<Vec<_>>();

            analyses.sort_by_key(|analysis| analysis.analysis.profit);
            for analysis in analyses {
                if analysis.analysis.profit < settings.min_crafting_profit
                    || analysis.analysis.velocity_info_nq.velocity
                        + analysis.analysis.velocity_info_hq.velocity
                        < settings.min_crafting_velocity
                {
                    continue;
                }
                write!(
                    &mut writer,
                    "{:<40}| {:<24}| {:<24}| {:<8}{:<8}{:<8}\n",
                    library.all_items[&analysis.analysis.item_id].name,
                    analysis.analysis.velocity_info_nq.to_string(),
                    analysis.analysis.velocity_info_hq.to_string(),
                    analysis.analysis.sell_price,
                    analysis.best_buy_price,
                    analysis.analysis.sell_price as i32 - analysis.best_buy_price as i32,
                )?;
            }
            write!(&mut writer, "\n")?;
        }

        Ok(())
    }
}
