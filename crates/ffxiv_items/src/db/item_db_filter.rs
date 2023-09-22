use std::time::Instant;

use anyhow::Result;
use const_format::formatcp;
use futures::TryStreamExt;
use sqlx::Row;

use super::{Filter, IngredientTable, ItemDB, ItemInfoTable, RecipeTable, UiCategoryTable};

impl ItemDB {
    fn get_query_string(clauses: &str) -> String {
        let mut query_string = SQL_SELECT.to_string();
        if clauses.contains("r.") {
            query_string = format!("{query_string} {SQL_JOIN_RECIPES}")
        };
        if clauses.contains("g.") {
            query_string = format!("{query_string} {SQL_JOIN_INGREDIENTS}")
        };
        if clauses.contains("c.") {
            query_string = format!("{query_string} {SQL_JOIN_UI_CATEGORIES}")
        };
        query_string
    }

    pub async fn ids_from_filter_str(&self, filter_str: &str) -> Result<Vec<u32>> {
        let start = Instant::now();
        let (clauses, binds) = Filter::apply_filter_str(filter_str);
        if clauses.is_empty() {
            return Ok(Vec::new());
        }

        let query_string = format!("{} WHERE {clauses}", Self::get_query_string(&clauses));
        let mut sql_query = sqlx::query(&query_string);
        for bind in binds {
            sql_query = sql_query.bind(bind);
        }

        let mut ids = Vec::new();
        let mut sql_query = sql_query.fetch(self);
        while let Some(row) = sql_query.try_next().await? {
            ids.push(row.get::<u32, _>(0));
        }

        log::debug!(target: "ffxiv_items", "Query for filter string ({} ids returned): {:.3}s", ids.len(), start.elapsed().as_secs_f32());
        Ok(ids)
    }
}

const SQL_SELECT: &str = formatcp!("SELECT DISTINCT i.id, i.name FROM {ITEM_TABLE_NAME} AS i");

const SQL_JOIN_RECIPES: &str = formatcp!("INNER JOIN {RECIPE_TABLE_NAME} AS r ON r.item_id = i.id");

const SQL_JOIN_INGREDIENTS: &str =
    formatcp!("INNER JOIN {INGREDIENT_TABLE_NAME} AS g ON g.recipe_id = r.id");

const SQL_JOIN_UI_CATEGORIES: &str =
    formatcp!("INNER JOIN {UI_CATEGORY_TABLE_NAME} AS c ON i.ui_category = c.id");

const ITEM_TABLE_NAME: &str = ItemInfoTable::SQL_TABLE_NAME;
const RECIPE_TABLE_NAME: &str = RecipeTable::SQL_TABLE_NAME;
const INGREDIENT_TABLE_NAME: &str = IngredientTable::SQL_TABLE_NAME;
const UI_CATEGORY_TABLE_NAME: &str = UiCategoryTable::SQL_TABLE_NAME;
