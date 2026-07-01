use std::time::Instant;

use crate::model::ModelMeta;
use crate::query::builder::QueryBuilder;
use crate::query::statement::BindArg;

use sqlx::Executor;
use tracing::debug;

// =========================================================================
// Macro — genera métodos de ejecución para un DB concreto.
// =========================================================================
macro_rules! impl_query_executor {
    ($db:ty) => {
        impl<'a, T: ModelMeta + Send + Unpin> QueryBuilder<'a, $db, T>
        where for<'r> T: sqlx::FromRow<'r, <$db as sqlx::Database>::Row>,
        {
            pub async fn fetch_all(self, executor: impl Executor<'_, Database = $db>) -> anyhow::Result<Vec<T>> {
                let start = Instant::now();
                let sql = self.build_sql();
                debug!(sql = %sql, value_count = self.values.len(), "fetch_all");

                let mut q = sqlx::query_as::<_, T>(&sql);
                for v in self.values {
                    q = match v {
                        BindArg::I64(v) => q.bind(v),
                        BindArg::F64(v) => q.bind(v),
                        BindArg::Text(v) => q.bind(v),
                        BindArg::Bool(v) => q.bind(v),
                    };
                }

                let result = q.fetch_all(executor).await;
                let elapsed = start.elapsed();

                match result {
                    Ok(rows) => {
                        debug!(count = rows.len(), elapsed_us = elapsed.as_micros() as u64, "fetch_all done");
                        Ok(rows)
                    }
                    Err(e) => {
                        debug!(error = %e, elapsed_us = elapsed.as_micros() as u64, "fetch_all failed");
                        Err(anyhow::anyhow!(e))
                    }
                }
            }

            pub async fn fetch_one(self, executor: impl Executor<'_, Database = $db>) -> anyhow::Result<T> {
                let start = Instant::now();
                let sql = self.build_sql();
                debug!(sql = %sql, value_count = self.values.len(), "fetch_one");

                let mut q = sqlx::query_as::<_, T>(&sql);
                for v in self.values {
                    q = match v {
                        BindArg::I64(v) => q.bind(v),
                        BindArg::F64(v) => q.bind(v),
                        BindArg::Text(v) => q.bind(v),
                        BindArg::Bool(v) => q.bind(v),
                    };
                }

                let result = q.fetch_one(executor).await;
                let elapsed = start.elapsed();

                match result {
                    Ok(row) => {
                        debug!(elapsed_us = elapsed.as_micros() as u64, "fetch_one done");
                        Ok(row)
                    }
                    Err(e) => {
                        debug!(error = %e, elapsed_us = elapsed.as_micros() as u64, "fetch_one failed");
                        Err(anyhow::anyhow!(e))
                    }
                }
            }

            pub async fn fetch_optional(self, executor: impl Executor<'_, Database = $db>) -> anyhow::Result<Option<T>> {
                let start = Instant::now();
                let sql = self.build_sql();
                debug!(sql = %sql, value_count = self.values.len(), "fetch_optional");

                let mut q = sqlx::query_as::<_, T>(&sql);
                for v in self.values {
                    q = match v {
                        BindArg::I64(v) => q.bind(v),
                        BindArg::F64(v) => q.bind(v),
                        BindArg::Text(v) => q.bind(v),
                        BindArg::Bool(v) => q.bind(v),
                    };
                }

                let result = q.fetch_optional(executor).await;
                let elapsed = start.elapsed();

                match result {
                    Ok(row) => {
                        debug!(found = row.is_some(), elapsed_us = elapsed.as_micros() as u64, "fetch_optional done");
                        Ok(row)
                    }
                    Err(e) => {
                        debug!(error = %e, elapsed_us = elapsed.as_micros() as u64, "fetch_optional failed");
                        Err(anyhow::anyhow!(e))
                    }
                }
            }
        }

        impl<'a, T: ModelMeta> QueryBuilder<'a, $db, T> {
            pub async fn execute(self, executor: impl Executor<'_, Database = $db>) -> anyhow::Result<u64> {
                let start = Instant::now();
                let sql = self.build_sql();
                debug!(sql = %sql, value_count = self.values.len(), "execute");

                let mut q = sqlx::query(&sql);
                for v in self.values {
                    q = match v {
                        BindArg::I64(v) => q.bind(v),
                        BindArg::F64(v) => q.bind(v),
                        BindArg::Text(v) => q.bind(v),
                        BindArg::Bool(v) => q.bind(v),
                    };
                }

                let result = q.execute(executor).await;
                let elapsed = start.elapsed();

                match result {
                    Ok(result) => {
                        let affected = result.rows_affected();
                        debug!(affected, elapsed_us = elapsed.as_micros() as u64, "execute done");
                        Ok(affected)
                    }
                    Err(e) => {
                        debug!(error = %e, elapsed_us = elapsed.as_micros() as u64, "execute failed");
                        Err(anyhow::anyhow!(e))
                    }
                }
            }
        }
    };
}

#[cfg(feature = "sqlite")]
impl_query_executor!(sqlx::Sqlite);

#[cfg(feature = "postgres")]
impl_query_executor!(sqlx::Postgres);
