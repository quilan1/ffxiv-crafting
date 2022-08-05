use std::collections::BTreeSet;

use axum::{http::StatusCode, response::IntoResponse, Form, Json};
use axum_macros::debug_handler;
use serde::{Deserialize, Serialize};

use crate::{
    library::{AnalysisFilters, Filter, Ingredient, RecursiveMarketBoardAnalysis, WorldInfo},
    universalis::Universalis,
    util::{item, item_name, library},
};

#[derive(Deserialize, Debug)]
pub struct CustomFilter {
    filters: String,
}

#[derive(Serialize, Debug)]
pub struct RecAnalysis {
    name: String,
    count: u32,
    buy_price: u32,
    best_buy_price: u32,
    buy_worlds: Vec<WorldInfo>,
    child_analyses: Vec<RecAnalysis>,
}

#[derive(Serialize, Debug)]
pub struct CustomOut {
    analyses: Vec<RecAnalysis>,
}

pub struct Custom;

impl Custom {
    #[debug_handler]
    pub async fn custom_filter(Form(payload): Form<CustomFilter>) -> impl IntoResponse {
        println!("GET custom_filter: Payload {payload:?}");

        let (top_ids, ids, filters) = get_ids_from_filters(payload.filters);
        let universalis = Universalis::get_mb_info_ids(ids.clone()).await.unwrap();
        let (analyses, _purchases) = get_analyses(top_ids, filters, &universalis);

        let mut json_analyses = Vec::new();
        for (index, (analysis, _filters)) in analyses.into_iter().enumerate() {
            if let Some(limit) = _filters.limit {
                if index >= limit as usize {
                    break;
                }
            }

            json_analyses.push(json_analysis(analysis));
        }

        (StatusCode::OK, Json(json_analyses))
    }
}

fn get_ids_from_filters(filters: String) -> (Vec<u32>, Vec<u32>, Vec<Filter>) {
    let item_list = library().all_items.items.values().collect::<Vec<_>>();
    let (items, filters) = Filter::apply_filters(item_list, &filters);

    fn push_ids(ids: &mut Vec<u32>, item_id: u32) {
        ids.push(item_id);
        if !library().all_recipes.contains_item_id(item_id) {
            return;
        }

        for input in &library().all_recipes[&item_id].inputs {
            push_ids(ids, input.item_id);
        }
    }

    let ids = items
        .iter()
        .map(|item| {
            let mut item_ids = Vec::new();
            push_ids(&mut item_ids, item.id);
            item_ids
        })
        .flatten()
        .filter(|item_id| !item(item_id).is_untradable)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();

    let items = items.into_iter().map(|item| item.id).collect::<Vec<_>>();

    (items, ids, filters)
}

fn get_analyses(
    ids: Vec<u32>,
    filters: Vec<Filter>,
    universalis: &Universalis,
) -> (
    Vec<(RecursiveMarketBoardAnalysis, AnalysisFilters)>,
    Vec<Ingredient>,
) {
    let mut purchases = Vec::new();
    let mut analyses = Vec::new();
    for item_id in ids {
        let mut analysis_filters = AnalysisFilters::new(&Vec::new(), &filters).unwrap();
        let multiplier = analysis_filters.count.unwrap_or(1);
        analysis_filters.always_top = true;

        let rec_analysis = match RecursiveMarketBoardAnalysis::analyze(
            item_id,
            &universalis,
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

    (analyses, purchases)
}

fn json_analysis(analysis: RecursiveMarketBoardAnalysis) -> RecAnalysis {
    let name = item_name(&analysis.ingredient).to_owned();
    let count = analysis.ingredient.count;
    let buy_price = if !analysis.children.is_empty() {
        analysis.analysis.buy_price
    } else {
        analysis.analysis.buy_price
        // analysis.best_buy_price
    };
    let best_buy_price = analysis.best_buy_price;
    // let buy_worlds = if !analysis.children.is_empty() {
    //     Vec::new()
    // } else {
    //     analysis.analysis.buy_worlds.clone()
    // };
    let buy_worlds = analysis.analysis.buy_worlds.clone();

    let child_analyses = if !analysis.children.is_empty() {
        analysis
            .children
            .into_iter()
            .map(|child| json_analysis(child))
            .collect::<Vec<_>>()
    } else {
        Vec::new()
    };

    RecAnalysis {
        name,
        count,
        buy_price,
        best_buy_price,
        buy_worlds,
        child_analyses,
    }
}
