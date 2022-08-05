use std::{collections::BTreeMap, fmt::Debug};

use serde::Serialize;

use crate::{
    cli::settings,
    library::{AsIngredient, Ingredient},
    universalis::{ItemListing, MarketBoardItemInfo, Universalis},
};

use super::{
    craft_list::{AnalysisFilters, QualityFilter},
    library,
};

#[derive(Default)]
pub struct VelocityAnalysis {
    pub velocity: f32,
    pub count: u32,
}

#[derive(Serialize, Default, Clone)]
pub struct WorldInfo {
    pub world: String,
    pub count: u32,
    pub price: u32,
}

#[derive(Default)]
pub struct MarketBoardAnalysis {
    pub item_id: u32,
    pub buy_price: u32,
    pub sell_price: u32,
    pub profit: i32,
    pub profit_margin: f32,
    pub velocity_info_nq: VelocityAnalysis,
    pub velocity_info_hq: VelocityAnalysis,
    pub buy_worlds: Vec<WorldInfo>,
}

impl MarketBoardAnalysis {
    pub fn from_item<I: AsIngredient>(
        ingredient: I,
        universalis: &Universalis,
        analysis_filters: &AnalysisFilters,
    ) -> Option<Self> {
        let ingredient = ingredient.as_ingredient();
        let item_id = ingredient.item_id;

        let valid_data_centers = universalis
            .data_centers
            .iter()
            .filter(|mb| mb.contains_key(&item_id))
            .collect::<Vec<_>>();

        if !universalis.homeworld.contains_key(&item_id) || valid_data_centers.is_empty() {
            return None;
        }

        let item_mb_homeworld = &universalis.homeworld[&item_id];
        let item_mb_data_centers = valid_data_centers
            .into_iter()
            .map(|data_center| &data_center[&item_id])
            .collect::<Vec<_>>();

        let velocity_info_nq = Self::velocity_info(item_mb_homeworld, analysis_filters, false);
        let velocity_info_hq = Self::velocity_info(item_mb_homeworld, analysis_filters, true);

        let sell_price = (f32::max(item_mb_homeworld.price_nq, item_mb_homeworld.price_hq)
            * ingredient.count as f32
            / 1.05) as u32;

        let mut listings = item_mb_data_centers
            .iter()
            .map(|mb| &mb.listings)
            .flatten()
            .collect::<Vec<_>>();
        listings.sort_by_key(|listing| listing.price);

        let mut buy_worlds = BTreeMap::<String, (u32, u32)>::new();
        let mut rem_count = ingredient.count;
        let mut buy_price = 0;
        for listing in listings {
            let used_count = u32::min(listing.count, rem_count);
            buy_price += used_count * listing.price;
            rem_count -= used_count;
            let mut entry = buy_worlds.entry(listing.world.clone()).or_default();
            entry.0 += listing.count;
            entry.1 = u32::max(entry.1, listing.price);
            if rem_count == 0 {
                break;
            }
        }
        let ceil_price = item_mb_data_centers
            .iter()
            .map(|mb| mb.price_nq.ceil() as i32)
            .max()
            .unwrap() as f32;
        buy_price += (rem_count as f32 * ceil_price) as u32;

        let mut buy_worlds = buy_worlds
            .into_iter()
            .map(|(world, (count, price))| WorldInfo {
                world,
                count,
                price,
            })
            .collect::<Vec<_>>();
        buy_worlds.sort_by_key(|info| info.price);

        let profit = sell_price as i32 - buy_price as i32;
        let profit_margin = profit as f32 / buy_price as f32;
        Some(Self {
            item_id,
            buy_price,
            sell_price,
            buy_worlds,
            profit,
            profit_margin,
            velocity_info_nq,
            velocity_info_hq,
        })
    }

    fn velocity_info(
        mb_item_info: &MarketBoardItemInfo,
        analysis_filters: &AnalysisFilters,
        is_hq: bool,
    ) -> VelocityAnalysis {
        if is_hq && analysis_filters.quality == QualityFilter::NQ {
            return VelocityAnalysis::default();
        }

        let is_nq_filter = analysis_filters.quality == QualityFilter::NQ;
        let price = if is_nq_filter {
            mb_item_info.price_avg
        } else if is_hq {
            f32::max(
                f32::max(mb_item_info.price_nq, mb_item_info.price_hq),
                mb_item_info.min_price_hq as f32,
            )
        } else {
            mb_item_info.price_nq
        };

        let listing_threshold = (price as f32 * settings().listings_ratio) as u32;
        let is_lower_listing = |listing: &&ItemListing| listing.price <= listing_threshold;
        let is_quality_listing = |listing: &&ItemListing| listing.is_hq == is_hq || is_nq_filter;

        let count = mb_item_info
            .listings
            .iter()
            .filter(is_lower_listing)
            .filter(is_quality_listing)
            .map(|listing| listing.count)
            .sum::<u32>();

        VelocityAnalysis {
            velocity: if is_nq_filter {
                mb_item_info.velocity_nq + mb_item_info.velocity_hq
            } else if is_hq {
                mb_item_info.velocity_hq
            } else {
                mb_item_info.velocity_nq
            },
            count,
        }
    }
}

impl ToString for VelocityAnalysis {
    fn to_string(&self) -> String {
        if self.velocity == 0.0 {
            if self.count == 0 {
                "".into()
            } else {
                format!("{:<8.1}{:<8}{:<8.0}", "", self.count, "",)
            }
        } else {
            let margin = 100.0 * (self.count as f32 - self.velocity) / self.velocity;
            let velocity_margin = if margin > 0.0 {
                format!("+ {margin:.0}%")
            } else {
                format!("- {:.0}%", -margin)
            };

            format!(
                "{:<8.1}{:<8}{:<8}",
                self.velocity, self.count, velocity_margin,
            )
        }
    }
}

pub struct RecursiveMarketBoardAnalysis {
    pub ingredient: Ingredient,
    pub analysis: MarketBoardAnalysis,
    pub best_buy_price: u32,
    pub children: Vec<Self>,
}

impl RecursiveMarketBoardAnalysis {
    pub fn analyze<I: AsIngredient>(
        ingredient: I,
        universalis: &Universalis,
        multiplier: u32,
        is_top: bool,
        analysis_filters: &AnalysisFilters,
    ) -> Option<Self> {
        let ingredient = ingredient.as_ingredient();
        let multiplier = ingredient.count * multiplier;
        let ingredient = Ingredient {
            item_id: ingredient.item_id,
            count: multiplier,
        };

        let analysis =
            match MarketBoardAnalysis::from_item(&ingredient, universalis, analysis_filters) {
                Some(analysis) => analysis,
                None => MarketBoardAnalysis {
                    item_id: ingredient.item_id,
                    buy_worlds: vec![WorldInfo {
                        world: "--Not on MB--".into(),
                        ..Default::default()
                    }],
                    ..Default::default()
                },
            };

        let mut rec_analysis = Self {
            ingredient: ingredient.clone(),
            best_buy_price: analysis.buy_price,
            analysis,
            children: Vec::new(),
        };

        match library().all_recipes.get(&ingredient.item_id) {
            None => {}
            Some(recipe) => {
                let mut total_child_cost = 0;
                let mut children = Vec::new();
                for input in &recipe.inputs {
                    let analysis = match Self::analyze(
                        input,
                        universalis,
                        (multiplier + recipe.output.count - 1) / recipe.output.count,
                        false,
                        analysis_filters,
                    ) {
                        None => continue,
                        Some(v) => v,
                    };
                    total_child_cost +=
                        u32::min(analysis.best_buy_price, analysis.analysis.buy_price);
                    children.push(analysis);
                }

                if is_top
                    || analysis_filters.always_top
                    || total_child_cost < rec_analysis.best_buy_price
                {
                    rec_analysis.best_buy_price = total_child_cost;
                    rec_analysis.children = children;
                    if !analysis_filters.always_top {
                        rec_analysis.analysis.buy_worlds = Vec::new();
                    }
                }
            }
        }

        Some(rec_analysis)
    }

    pub fn all_purchases(&self) -> Vec<Ingredient> {
        if self.children.is_empty() {
            return vec![self.ingredient.clone()];
        }

        let mut leaf_nodes = Vec::new();
        for child in &self.children {
            leaf_nodes.extend(child.all_purchases());
        }

        leaf_nodes
    }
}

impl Debug for WorldInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}x <{}", self.world, self.count, self.price)
    }
}
