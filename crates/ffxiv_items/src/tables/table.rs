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

            #[allow(dead_code)]
            pub async fn drop(&self) -> Result<()> {
                let sql_drop = format!("DROP TABLE IF EXISTS {SQL_TABLE_NAME}");
                sqlx::query(&sql_drop).execute(&*self.db).await?;
                Ok(())
            }

            pub async fn is_empty(&self) -> Result<bool> {
                let sql_empty = format!("SELECT COUNT(id) FROM {SQL_TABLE_NAME}");
                Ok(0 == sqlx::query_scalar::<_, i64>(&sql_empty)
                    .fetch_one(&*self.db)
                    .await?)
            }
        }
    };
}
