macro_rules! make_struct {
    (@ $table:tt) => {
        pub struct $table<'a> {
            db: &'a ItemDB,
        }
    };

    (@ $table:tt, $f:tt) => {
        pub struct $table<'a, F: $f> {
            db: &'a ItemDB,
            _marker_f: std::marker::PhantomData<fn() -> F>,
        }
    };
}

macro_rules! make_table_impl {
    (@ $table:tt) => {
        impl<'a> $table<'a> {
            pub fn new<'b: 'a>(db: &'b ItemDB) -> Self {
                Self { db }
            }

            #[allow(dead_code)]
            pub const SQL_TABLE_NAME: &'static str = SQL_TABLE_NAME;

            pub async fn create(&self) -> Result<()> {
                sqlx::query(&$crate::tables::strip_whitespace(SQL_CREATE))
                    .execute(&*self.db)
                    .await?;
                Ok(())
            }

            #[allow(dead_code)]
            pub async fn drop(&self) -> Result<()> {
                let sql_drop = format!("DROP TABLE IF EXISTS {SQL_TABLE_NAME}");
                sqlx::query(&sql_drop).execute(&*self.db).await?;
                Ok(())
            }

            pub async fn is_empty(&self) -> Result<bool> {
                let sql_empty = format!("SELECT COUNT(*) FROM {SQL_TABLE_NAME}");
                Ok(0 == sqlx::query_scalar::<_, i64>(&sql_empty)
                    .persistent(true)
                    .fetch_one(&*self.db)
                    .await?)
            }
        }
    };
}

macro_rules! make_table_builder_impl {
    (@ $table:tt, $f:tt) => {
        impl<'a, F: $f> $table<'a, F> {
            pub fn new<'b: 'a>(db: &'b ItemDB) -> Self {
                Self {
                    db,
                    _marker_f: std::marker::PhantomData,
                }
            }
        }
    };
}

#[macro_export]
macro_rules! impl_table {
    ($table:tt) => {
        make_struct!(@ $table);
        make_table_impl!(@ $table);
    };
}

macro_rules! impl_table_builder {
    ($table:tt, $f:tt) => {
        make_struct!(@ $table, $f);
        make_table_builder_impl!(@ $table, $f);
    }
}
