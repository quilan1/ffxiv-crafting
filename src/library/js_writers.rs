use std::{
    fs::{DirBuilder, File},
    io::{BufWriter, Write},
    path::Path,
};

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{
    cli::{settings, RunMode},
    library::market_board_analysis::VelocityAnalysis,
    universalis::Universalis,
    util::item_name,
};

use super::{
    craft_list::{AnalysisFilters, CraftList},
    library, RecursiveMarketBoardAnalysis,
};

// struct MarketboardData {
//     crafts:
// }

#[derive(Serialize, Deserialize)]
struct CraftJsData {
    groups: Vec<CraftJsGroup>,
}

#[derive(Serialize, Deserialize)]
struct CraftJsGroup {
    name: String,
    crafts: Vec<CraftJsCraft>,
}

#[derive(Serialize, Deserialize)]
struct CraftJsCraft {
    item_name: String,
    velocity_nq: CraftJsVelocityInfo,
    velocity_hq: CraftJsVelocityInfo,
    local_sell_price: u32,
    best_buy_price: u32,
    profit: i32,
}

#[derive(Serialize, Deserialize)]
struct CraftJsVelocityInfo {
    velocity: f32,
    count: u32,
}

pub struct JsWriter;

impl JsWriter {
    pub fn write_all(universalis: &Universalis) -> Result<()> {
        DirBuilder::new().recursive(true).create("./js")?;

        let run_mode = &settings().run_mode;
        if [RunMode::OnlyCrafting, RunMode::All].contains(run_mode) {
            Self::write_crafts_js(&library().all_crafts, "./js/crafts.json", universalis)?;
        }
        // if [RunMode::OnlyCustom, RunMode::All].contains(&run_mode) {
        //     self.all_custom_crafts.write_custom_to_file(
        //         "./out/custom.txt",
        //         universalis,
        //         settings,
        //     )?;
        // }
        // if [RunMode::OnlyGathering, RunMode::All].contains(&run_mode) {
        //     self.all_gathering
        //         .write_to_file("./out/gathering.txt", universalis, settings)?;
        // }

        Ok(())
    }

    pub fn write_marketboard_js<P: AsRef<Path>>() -> Result<()> {
        Ok(())
    }

    pub fn write_crafts_js<P: AsRef<Path>>(
        craft_list: &CraftList,
        path: P,
        universalis: &Universalis,
    ) -> Result<()> {
        let writer = &mut BufWriter::new(File::create(path.as_ref())?);

        fn make_velocity_info(velocity_info: &VelocityAnalysis) -> CraftJsVelocityInfo {
            let &VelocityAnalysis { velocity, count } = velocity_info;
            CraftJsVelocityInfo { velocity, count }
        }

        let mut js_groups = Vec::new();

        for group in &craft_list.craft_groups {
            let analysis_filters = AnalysisFilters::from_filters(&group.filters)?;
            let crafts = group
                .crafts
                .iter()
                .filter_map(|craft| {
                    match RecursiveMarketBoardAnalysis::analyze(
                        craft.item_id,
                        universalis,
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
                .map(|analysis| CraftJsCraft {
                    item_name: item_name(&analysis).to_string(),
                    velocity_nq: make_velocity_info(&analysis.analysis.velocity_info_nq),
                    velocity_hq: make_velocity_info(&analysis.analysis.velocity_info_hq),
                    local_sell_price: analysis.analysis.sell_price,
                    best_buy_price: analysis.best_buy_price,
                    profit: analysis.analysis.profit,
                })
                .collect::<Vec<_>>();

            js_groups.push(CraftJsGroup {
                name: group.heading.clone(),
                crafts,
            });
        }

        // write!(writer, "{}", serde_json::to_string(&CraftJsData{ groups: js_groups })?)?;
        write!(writer, "{}", serde_json::to_string(&js_groups)?)?;

        Ok(())
    }
}
