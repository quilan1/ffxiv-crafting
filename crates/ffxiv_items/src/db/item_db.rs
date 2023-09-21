use std::{io::Cursor, sync::Arc};

use anyhow::Result;
use futures::try_join;
use itertools::Itertools;
use sqlx::MySqlPool;
use tuple_conv::RepeatedTuple;

use crate::{parsers, CsvContent, ItemInfo, Recipe, RecipeLevelInfo};

use super::{IngredientTable, InputIdsTable, ItemInfoTable, RecipeTable, UiCategoryTable};

#[derive(Debug)]
pub struct ItemDB {
    pub(super) pool: MySqlPool,
}

struct Tables<'a> {
    items: ItemInfoTable<'a>,
    recipes: RecipeTable<'a>,
    ingredients: IngredientTable<'a>,
    input_ids: InputIdsTable<'a>,
    ui_categories: UiCategoryTable<'a>,
}

////////////////////////////////////////////////////////////

impl ItemDB {
    pub async fn connect<S: AsRef<str>>(conn_string: S) -> Result<Self> {
        let pool = MySqlPool::connect(conn_string.as_ref()).await?;
        Ok(Self { pool })
    }

    pub async fn initialize(&self) -> Result<()> {
        self.create_tables().await?;
        self.fill_tables().await?;
        Ok(())
    }

    fn tables(&self) -> Tables<'_> {
        Tables {
            items: ItemInfoTable::new(self),
            recipes: RecipeTable::new(self),
            ingredients: IngredientTable::new(self),
            input_ids: InputIdsTable::new(self),
            ui_categories: UiCategoryTable::new(self),
        }
    }

    async fn create_tables(&self) -> Result<()> {
        self.tables().create().await
    }

    async fn has_empty_table(&self) -> Result<bool> {
        self.tables().has_empty_table().await
    }

    async fn fill_tables(&self) -> Result<()> {
        if !self.has_empty_table().await? {
            return Ok(());
        }

        let csv_content = CsvContent::download().await?;
        let items = parsers::ItemList::from_reader(csv_content.item)?;
        let recipes = Self::parse_recipes(csv_content.recipe, csv_content.recipe_level)?;
        let ui_categories = parsers::UiCategoryList::from_reader(csv_content.item_ui_category)?;

        self.tables()
            .fill_tables(&items, &recipes, &ui_categories)
            .await
    }

    pub async fn get_ids_from_filters(&self) -> Result<(Vec<u32>, Vec<u32>)> {
        let top_ids = self
            .ids_from_filter_str(":rlevel 90,:cat Metal|Lumber|Leather|Stone|Cloth|Reagent")
            .await?;
        let all_ids = InputIdsTable::new(self).by_item_ids(&top_ids).await?;
        Ok((top_ids, all_ids))
    }

    pub async fn items_from_ids(self: &Arc<Self>, ids: &[u32]) -> Result<Vec<ItemInfo>> {
        ItemInfoTable::new(self).by_item_ids(ids).await
    }

    fn parse_recipes(
        recipes: Cursor<String>,
        recipe_levels: Cursor<String>,
    ) -> Result<Vec<Recipe>> {
        let all_recipes = parsers::RecipeList::from_reader(recipes)?;
        let all_recipe_levels = parsers::RecipeLevelTable::from_reader(recipe_levels)?;

        let mut recipes = Vec::new();
        for recipe_parsed in all_recipes.0.into_values() {
            let level_info = all_recipe_levels
                .0
                .get(&recipe_parsed.level_id)
                .map(|level_info| RecipeLevelInfo {
                    level: level_info.level,
                    stars: level_info.stars,
                })
                .unwrap();

            recipes.push(Recipe {
                output: recipe_parsed.output,
                inputs: recipe_parsed.inputs,
                level: level_info.level,
                stars: level_info.stars,
            });
        }

        Ok(recipes)
    }
}

////////////////////////////////////////////////////////////

impl Tables<'_> {
    async fn create(&self) -> Result<()> {
        try_join!(
            self.items.create(),
            self.recipes.create(),
            self.ingredients.create(),
            self.input_ids.create(),
            self.ui_categories.create()
        )?;
        Ok(())
    }

    async fn fill_tables(
        &self,
        items: &parsers::ItemList,
        recipes: &[Recipe],
        ui_categories: &parsers::UiCategoryList,
    ) -> Result<()> {
        try_join!(
            self.items.initialize(items),
            self.recipes.initialize(recipes),
            self.ingredients.initialize(recipes),
            self.input_ids.initialize(items, recipes),
            self.ui_categories.initialize(ui_categories)
        )?;
        Ok(())
    }

    async fn has_empty_table(&self) -> Result<bool> {
        let empties: Vec<bool> = try_join!(
            self.items.is_empty(),
            self.recipes.is_empty(),
            self.ingredients.is_empty(),
            self.input_ids.is_empty(),
            self.ui_categories.is_empty(),
        )?
        .to_vec();

        Ok(empties.into_iter().contains(&true))
    }
}

////////////////////////////////////////////////////////////

mod executor {
    use std::result::Result;

    use futures::{future::BoxFuture, stream::BoxStream};
    use itertools::Either;
    use sqlx::{database::HasStatement, Database, Describe, Error, Execute, Executor, MySql};

    use super::ItemDB;

    impl<'c> Executor<'c> for &ItemDB {
        type Database = MySql;

        fn fetch_many<'e, 'q: 'e, E: 'q>(
            self,
            query: E,
        ) -> BoxStream<
            'e,
            Result<
                Either<
                    <Self::Database as Database>::QueryResult,
                    <Self::Database as Database>::Row,
                >,
                Error,
            >,
        >
        where
            'e: 'e,
            E: Execute<'q, Self::Database>,
        {
            self.pool.fetch_many(query)
        }

        fn fetch_optional<'e, 'q: 'e, E: 'q>(
            self,
            query: E,
        ) -> BoxFuture<'e, Result<Option<<Self::Database as Database>::Row>, Error>>
        where
            'e: 'e,
            E: Execute<'q, Self::Database>,
        {
            self.pool.fetch_optional(query)
        }

        fn prepare_with<'e, 'q: 'e>(
            self,
            sql: &'q str,
            parameters: &'e [<Self::Database as Database>::TypeInfo],
        ) -> BoxFuture<'e, Result<<Self::Database as HasStatement<'q>>::Statement, Error>>
        where
            'e: 'e,
        {
            self.pool.prepare_with(sql, parameters)
        }

        fn describe<'e, 'q: 'e>(
            self,
            query: &'q str,
        ) -> BoxFuture<'e, Result<Describe<Self::Database>, Error>> {
            self.pool.describe(query)
        }
    }
}
