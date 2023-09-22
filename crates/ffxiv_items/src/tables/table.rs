#[macro_export]
macro_rules! impl_table {
    ($table:tt) => {
        pub struct $table<'a> {
            db: &'a ItemDB,
        }

        impl<'a> $table<'a> {
            #[allow(dead_code)]
            pub const SQL_TABLE_NAME: &str = SQL_TABLE_NAME;

            pub fn new<'b: 'a>(db: &'b ItemDB) -> Self {
                Self { db }
            }

            pub async fn create(&self) -> Result<()> {
                sqlx::query(SQL_CREATE).execute(&*self.db).await?;
                Ok(())
            }

            pub async fn is_empty(&self) -> Result<bool> {
                Ok(0 == sqlx::query_scalar::<_, i64>(SQL_EMPTY)
                    .fetch_one(&*self.db)
                    .await?)
            }
        }
    };
}
