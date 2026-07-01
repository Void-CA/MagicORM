use crate::model::ModelMeta;
use crate::query::builder::QueryBuilder;
use crate::query::statement::BindArg;

use sqlx::Executor;

// =========================================================================
// Macro — genera métodos de ejecución para un DB concreto.
// Necesario porque sqlx::Encode y sqlx::Type se implementan por DB concreto,
// no para Database genérico.
// =========================================================================
macro_rules! impl_query_executor {
    ($db:ty) => {
        impl<'a, T: ModelMeta + Send + Unpin> QueryBuilder<'a, $db, T>
        where for<'r> T: sqlx::FromRow<'r, <$db as sqlx::Database>::Row>,
        {
            pub async fn fetch_all(self, executor: impl Executor<'_, Database = $db>) -> anyhow::Result<Vec<T>> {
                let sql = self.build_sql();
                let mut q = sqlx::query_as::<_, T>(&sql);
                for v in self.values {
                    q = match v {
                        BindArg::I64(v) => q.bind(v),
                        BindArg::F64(v) => q.bind(v),
                        BindArg::Text(v) => q.bind(v),
                        BindArg::Bool(v) => q.bind(v),
                    };
                }
                q.fetch_all(executor).await.map_err(|e| anyhow::anyhow!(e))
            }

            pub async fn fetch_one(self, executor: impl Executor<'_, Database = $db>) -> anyhow::Result<T> {
                let sql = self.build_sql();
                let mut q = sqlx::query_as::<_, T>(&sql);
                for v in self.values {
                    q = match v {
                        BindArg::I64(v) => q.bind(v),
                        BindArg::F64(v) => q.bind(v),
                        BindArg::Text(v) => q.bind(v),
                        BindArg::Bool(v) => q.bind(v),
                    };
                }
                q.fetch_one(executor).await.map_err(|e| anyhow::anyhow!(e))
            }

            pub async fn fetch_optional(self, executor: impl Executor<'_, Database = $db>) -> anyhow::Result<Option<T>> {
                let sql = self.build_sql();
                let mut q = sqlx::query_as::<_, T>(&sql);
                for v in self.values {
                    q = match v {
                        BindArg::I64(v) => q.bind(v),
                        BindArg::F64(v) => q.bind(v),
                        BindArg::Text(v) => q.bind(v),
                        BindArg::Bool(v) => q.bind(v),
                    };
                }
                q.fetch_optional(executor).await.map_err(|e| anyhow::anyhow!(e))
            }
        }

        impl<'a, T: ModelMeta> QueryBuilder<'a, $db, T> {
            pub async fn execute(self, executor: impl Executor<'_, Database = $db>) -> anyhow::Result<u64> {
                let sql = self.build_sql();
                let mut q = sqlx::query(&sql);
                for v in self.values {
                    q = match v {
                        BindArg::I64(v) => q.bind(v),
                        BindArg::F64(v) => q.bind(v),
                        BindArg::Text(v) => q.bind(v),
                        BindArg::Bool(v) => q.bind(v),
                    };
                }
                let result = q.execute(executor).await.map_err(|e| anyhow::anyhow!(e))?;
                Ok(result.rows_affected())
            }
        }
    };
}

#[cfg(feature = "sqlite")]
impl_query_executor!(sqlx::Sqlite);

#[cfg(feature = "postgres")]
impl_query_executor!(sqlx::Postgres);
