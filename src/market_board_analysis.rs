use anyhow::Result;
use std::io::Write;

use crate::{
    library::{AsIngredient, Ingredient, Library},
    universalis::{ItemListing, MarketBoardItemInfo, Universalis},
    Settings,
};

#[derive(Default)]
pub struct VelocityAnalysis {
    pub velocity: f32,
    pub count: u32,
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
    pub buy_worlds: Vec<(String, u32, u32)>,
}

impl MarketBoardAnalysis {
    pub fn from_item<I: AsIngredient>(
        ingredient: I,
        universalis: &Universalis,
        settings: &Settings,
    ) -> Option<Self> {
        let ingredient = ingredient.as_ingredient();
        let item_id = ingredient.item_id;

        if !universalis.homeworld.contains_key(&item_id)
            || !universalis.data_center.contains_key(&item_id)
        {
            return None;
        }

        let item_mb_homeworld = &universalis.homeworld[&item_id];
        let item_mb_data_center = &universalis.data_center[&item_id];

        let velocity_info_nq = Self::velocity_info(item_mb_homeworld, settings, false);
        let velocity_info_hq = Self::velocity_info(item_mb_homeworld, settings, true);

        let sell_price = (f32::max(item_mb_homeworld.price, item_mb_homeworld.price_hq)
            * ingredient.count as f32) as u32;

        let mut listings = item_mb_data_center.listings.clone();
        listings.sort_by_key(|listing| listing.price);

        let mut buy_worlds = Vec::new();
        let mut rem_count = ingredient.count;
        let mut buy_price = 0;
        for listing in listings {
            let used_count = u32::min(listing.count, rem_count);
            buy_price += used_count * listing.price;
            rem_count -= used_count;
            buy_worlds.push((listing.world, listing.count, listing.price));
            if rem_count == 0 {
                break;
            }
        }
        buy_price += (rem_count as f32 * item_mb_data_center.price.ceil()) as u32;

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
        settings: &Settings,
        is_hq: bool,
    ) -> VelocityAnalysis {
        let price = if is_hq {
            f32::max(
                f32::max(mb_item_info.price, mb_item_info.price_hq),
                mb_item_info.min_price_hq as f32,
            )
        } else {
            mb_item_info.price
        };

        let listing_threshold = (price as f32 * settings.listings_ratio) as u32;
        let is_lower_listing = |listing: &ItemListing| listing.price <= listing_threshold;
        let is_quality_listing = |listing: &ItemListing| listing.is_hq == is_hq;

        let count = mb_item_info
            .listings
            .iter()
            .cloned()
            .filter(is_lower_listing)
            .filter(is_quality_listing)
            .map(|listing| listing.count)
            .sum::<u32>();

        VelocityAnalysis {
            velocity: if is_hq {
                mb_item_info.velocity_hq
            } else {
                mb_item_info.velocity
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
        library: &Library,
        universalis: &Universalis,
        settings: &Settings,
        multiplier: u32,
        is_top: bool,
    ) -> Option<Self> {
        let ingredient = ingredient.as_ingredient();
        let multiplier = ingredient.count * multiplier;
        let ingredient = Ingredient {
            item_id: ingredient.item_id,
            count: multiplier,
        };

        let analysis = match MarketBoardAnalysis::from_item(&ingredient, universalis, settings) {
            Some(analysis) => analysis,
            None => MarketBoardAnalysis {
                item_id: ingredient.item_id,
                buy_worlds: vec![("--Not on MB--".into(), 0, 0)],
                ..Default::default()
            },
        };

        let mut rec_analysis = Self {
            ingredient: ingredient.clone(),
            best_buy_price: analysis.buy_price,
            analysis,
            children: Vec::new(),
        };

        match library.all_recipes.get(&ingredient.item_id) {
            None => {}
            Some(recipe) => {
                let mut total_child_cost = 0;
                let mut children = Vec::new();
                for input in &recipe.inputs {
                    let analysis = match Self::analyze(
                        input,
                        library,
                        universalis,
                        settings,
                        (multiplier + recipe.output.count - 1) / recipe.output.count,
                        false,
                    ) {
                        None => continue,
                        Some(v) => v,
                    };
                    total_child_cost += analysis.best_buy_price;
                    children.push(analysis);
                }

                if is_top || total_child_cost < rec_analysis.best_buy_price {
                    rec_analysis.best_buy_price = total_child_cost;
                    rec_analysis.children = children;
                    rec_analysis.analysis.buy_worlds = Vec::new();
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

    pub fn write<W: Write>(&self, writer: &mut W, library: &Library) -> Result<()> {
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
