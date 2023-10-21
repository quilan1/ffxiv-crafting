use std::marker::PhantomData;

use anyhow::Result;
use futures::try_join;
use mock_traits::FileDownloader;
use sqlx::MySqlPool;
use tuple_conv::RepeatedTuple;

use crate::tables::{
    IngredientTable, InputIdsTable, ItemInfoTable, ItemInfoTableBuilder, RecipeTable,
    RecipeTableBuilder, UiCategoryTable, UiCategoryTableBuilder, UpdateTable,
};

/// The database used to look up information about items.
///
/// # Example
///
/// ```rust
/// use ffxiv_items::ItemDB;
/// use mock_traits::ReqwestDownloader;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn Error>> {
///     let db = ItemDB::connect(ITEM_DB_CONN).await?;
///     db.initialize<ReqwestDownloader>().await?;
///     let info = db.all_info_from_query(":name ^Rarefied, :rlevel 61|69").await?;
///     let (top_level_ids, associated_ids, item_info) = info;
/// }
/// ```
#[derive(Debug)]
pub struct ItemDB {
    pub(super) pool: MySqlPool,
}

struct Tables<'a, F: FileDownloader> {
    items: ItemInfoTable<'a>,
    items_builder: ItemInfoTableBuilder<'a, F>,
    recipes: RecipeTable<'a>,
    recipes_builder: RecipeTableBuilder<'a, F>,
    ingredients: IngredientTable<'a>,
    input_ids: InputIdsTable<'a>,
    ui_categories: UiCategoryTable<'a>,
    ui_categories_builder: UiCategoryTableBuilder<'a, F>,
    update_table: UpdateTable<'a>,
    marker_f: PhantomData<fn() -> F>,
}

////////////////////////////////////////////////////////////

impl ItemDB {
    /// Connects to the database with a connection string.
    /// The connection string should likely be in the format of:
    ///
    /// `mysql://<user>:<password>@<server>:<port>/<database>`
    pub async fn connect<S: AsRef<str>>(conn_string: S) -> Result<Self> {
        let pool = MySqlPool::connect(conn_string.as_ref()).await?;
        Ok(Self { pool })
    }

    /// Ensures the item database is either created or updated.
    ///
    /// The files are pulled from <https://www.github.com/xivapi/ffxiv-datamining>.
    /// Additionally, github commit metadata is checked, but is heavily rate
    /// limited to 60 attempts per IP per some time period. If this fails due to
    /// excessive rate limiting, it's assumed that the data is up-to-date.
    /// If you'd like to ensure the database is updated, you can simply delete
    /// the database or drop the tables.
    pub async fn initialize<F: FileDownloader>(&self) -> Result<bool> {
        let tables = self.tables::<F>();
        if cfg!(not(test)) {
            // We're going to swallow errors with github, wrt: rate limiting
            let _ = tables.check_updated_github().await;
        }
        tables.create().await
    }

    fn tables<F: FileDownloader>(&self) -> Tables<'_, F> {
        Tables {
            items: ItemInfoTable::new(self),
            items_builder: ItemInfoTableBuilder::new(self),
            recipes: RecipeTable::new(self),
            recipes_builder: RecipeTableBuilder::new(self),
            ingredients: IngredientTable::new(self),
            input_ids: InputIdsTable::new(self),
            ui_categories: UiCategoryTable::new(self),
            ui_categories_builder: UiCategoryTableBuilder::new(self),
            update_table: UpdateTable::new(self),
            marker_f: PhantomData,
        }
    }
}

////////////////////////////////////////////////////////////

impl<F: FileDownloader> Tables<'_, F> {
    async fn check_updated_github(&self) -> Result<()> {
        let last_updated_github = try_join!(
            self.items_builder.last_updated_github(),
            self.recipes_builder.last_updated_github(),
            self.ui_categories_builder.last_updated_github(),
        )?
        .to_vec()
        .into_iter()
        .max()
        .unwrap();

        let mut is_new_update = false;
        self.update_table.create().await?;
        if self.update_table.is_empty().await? {
            self.update_table.insert(&last_updated_github).await?;
            is_new_update = true;
        }

        let last_updated_db = self.update_table.last_updated().await?;
        is_new_update |= last_updated_github > last_updated_db;
        if is_new_update {
            try_join!(
                self.update_table.update(&last_updated_github),
                self.items.drop(),
                self.recipes.drop(),
                self.ui_categories.drop(),
                self.ingredients.drop(),
                self.input_ids.drop(),
            )?;
        }

        Ok(())
    }

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
            self.items_builder.initialize().await?;
        }
        Ok(is_empty)
    }

    async fn create_ui_categories(&self) -> Result<bool> {
        self.ui_categories.create().await?;
        let is_empty = self.ui_categories.is_empty().await?;
        if is_empty {
            self.ui_categories_builder.initialize().await?;
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
            RecipeTableBuilder::<'_, F>::download_recipe_info(),
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
            self.recipes_builder.initialize(&recipes),
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
