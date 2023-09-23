use anyhow::Result;
use futures::try_join;
use sqlx::MySqlPool;
use tuple_conv::RepeatedTuple;

use crate::tables::{IngredientTable, InputIdsTable, ItemInfoTable, RecipeTable, UiCategoryTable};

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

    pub async fn initialize(&self) -> Result<bool> {
        self.tables().create().await
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
}

////////////////////////////////////////////////////////////

impl Tables<'_> {
    async fn create(&self) -> Result<bool> {
        let is_empty = try_join!(
            self.create_items(),
            self.create_ui_categories(),
            self.create_recipes(),
        )?
        .to_vec()
        .into_iter()
        .any(|v| v);
        Ok(is_empty)
    }

    async fn create_items(&self) -> Result<bool> {
        self.items.create().await?;
        let is_empty = self.items.is_empty().await?;
        if is_empty {
            self.items.initialize().await?;
        }
        Ok(is_empty)
    }

    async fn create_ui_categories(&self) -> Result<bool> {
        self.ui_categories.create().await?;
        let is_empty = self.ui_categories.is_empty().await?;
        if is_empty {
            self.ui_categories.initialize().await?;
        }
        Ok(is_empty)
    }

    async fn create_recipes(&self) -> Result<bool> {
        let is_empty = try_join!(
            {
                self.recipes.create().await?;
                self.recipes.is_empty()
            },
            {
                self.ingredients.create().await?;
                self.ingredients.is_empty()
            },
            {
                self.input_ids.create().await?;
                self.input_ids.is_empty()
            },
        )?
        .to_vec()
        .into_iter()
        .any(|v| v);

        if !is_empty {
            return Ok(false);
        }

        let (recipes, _, _, _) = try_join!(
            RecipeTable::download_recipe_info(),
            {
                self.recipes.drop().await?;
                self.recipes.create()
            },
            {
                self.ingredients.drop().await?;
                self.ingredients.create()
            },
            {
                self.input_ids.drop().await?;
                self.input_ids.create()
            }
        )?;

        try_join!(
            self.recipes.initialize(&recipes),
            self.ingredients.initialize(&recipes),
            self.input_ids.initialize(&recipes),
        )?;

        Ok(true)
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
