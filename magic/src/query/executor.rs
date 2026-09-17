use std::time::Instant;

use crate::model::ModelMeta;
use crate::query::builder::QueryBuilder;
use crate::query::statement::BindArg;

use futures_core::stream::BoxStream;
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
                    BindArg::Null => q.bind(None::<i64>),
                    BindArg::I64(v) => q.bind(v),
                    BindArg::F64(v) => q.bind(v),
                    BindArg::Text(v) => q.bind(v),
                    BindArg::Bool(v) => q.bind(v),
                    BindArg::Uuid(v) => q.bind(v),
                    BindArg::Blob(v) => q.bind(v),
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

            /// Stream results lazily without loading all into memory.
            ///
            /// Returns a `BoxStream` that yields `Result<T, sqlx::Error>` items.
            /// The stream borrows the executor, so the executor must outlive the stream.
            ///
            /// # Example
            /// ```rust,ignore
            /// use futures::StreamExt;
            ///
            /// # async fn example(pool: sqlx::SqlitePool) -> anyhow::Result<()> {
            /// let mut stream = User::query()
            ///     .filter("active", "=", true)
            ///     .stream(&pool)?;
            ///
            /// while let Some(row) = stream.next().await {
            ///     let user = row?;
            ///     // process user...
            /// }
            /// # Ok(())
            /// # }
            /// ```
            pub fn stream<'e, 'c: 'e, E: 'e>(
                self,
                executor: E,
            ) -> Result<BoxStream<'e, Result<T, sqlx::Error>>, sqlx::Error>
            where
                E: Executor<'c, Database = $db>,
                T: 'e,
            {
                let sql = self.build_sql();
                debug!(sql = %sql, value_count = self.values.len(), "stream");

                // Leak the SQL string so it lives for 'static, which is required
                // because the stream needs to own the SQL. This is acceptable
                // because the string is small and the stream is short-lived.
                let sql: &'static str = Box::leak(sql.into_boxed_str());

                let mut q = sqlx::query_as::<_, T>(sql);
                for v in self.values {
                    q = match v {
                    BindArg::Null => q.bind(None::<i64>),
                    BindArg::I64(v) => q.bind(v),
                    BindArg::F64(v) => q.bind(v),
                    BindArg::Text(v) => q.bind(v),
                    BindArg::Bool(v) => q.bind(v),
                    BindArg::Uuid(v) => q.bind(v),
                    BindArg::Blob(v) => q.bind(v),
                };
                }

                Ok(q.fetch(executor))
            }

            pub async fn fetch_one(self, executor: impl Executor<'_, Database = $db>) -> anyhow::Result<T> {
                let start = Instant::now();
                let sql = self.build_sql();
                debug!(sql = %sql, value_count = self.values.len(), "fetch_one");

                let mut q = sqlx::query_as::<_, T>(&sql);
                for v in self.values {
                    q = match v {
                    BindArg::Null => q.bind(None::<i64>),
                    BindArg::I64(v) => q.bind(v),
                    BindArg::F64(v) => q.bind(v),
                    BindArg::Text(v) => q.bind(v),
                    BindArg::Bool(v) => q.bind(v),
                    BindArg::Uuid(v) => q.bind(v),
                    BindArg::Blob(v) => q.bind(v),
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
                    BindArg::Null => q.bind(None::<i64>),
                    BindArg::I64(v) => q.bind(v),
                    BindArg::F64(v) => q.bind(v),
                    BindArg::Text(v) => q.bind(v),
                    BindArg::Bool(v) => q.bind(v),
                    BindArg::Uuid(v) => q.bind(v),
                    BindArg::Blob(v) => q.bind(v),
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
                    BindArg::Null => q.bind(None::<i64>),
                    BindArg::I64(v) => q.bind(v),
                    BindArg::F64(v) => q.bind(v),
                    BindArg::Text(v) => q.bind(v),
                    BindArg::Bool(v) => q.bind(v),
                    BindArg::Uuid(v) => q.bind(v),
                    BindArg::Blob(v) => q.bind(v),
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

            /// Execute count query and return i64
            pub async fn fetch_count(self, executor: impl Executor<'_, Database = $db>) -> anyhow::Result<i64> {
                let start = Instant::now();
                let sql = self.build_sql();
                debug!(sql = %sql, value_count = self.values.len(), "fetch_count");

                let mut q = sqlx::query_scalar::<_, i64>(&sql);
                for v in self.values {
                    q = match v {
                    BindArg::Null => q.bind(None::<i64>),
                    BindArg::I64(v) => q.bind(v),
                    BindArg::F64(v) => q.bind(v),
                    BindArg::Text(v) => q.bind(v),
                    BindArg::Bool(v) => q.bind(v),
                    BindArg::Uuid(v) => q.bind(v),
                    BindArg::Blob(v) => q.bind(v),
                };
                }

                let result = q.fetch_one(executor).await;
                let elapsed = start.elapsed();

                match result {
                    Ok(count) => {
                        debug!(count, elapsed_us = elapsed.as_micros() as u64, "fetch_count done");
                        Ok(count)
                    }
                    Err(e) => {
                        debug!(error = %e, elapsed_us = elapsed.as_micros() as u64, "fetch_count failed");
                        Err(anyhow::anyhow!(e))
                    }
                }
            }

            /// Execute exists query and return bool
            pub async fn fetch_exists(self, executor: impl Executor<'_, Database = $db>) -> anyhow::Result<bool> {
                let start = Instant::now();
                let sql = self.build_sql();
                debug!(sql = %sql, value_count = self.values.len(), "fetch_exists");

                let mut q = sqlx::query_scalar::<_, i64>(&sql);
                for v in self.values {
                    q = match v {
                    BindArg::Null => q.bind(None::<i64>),
                    BindArg::I64(v) => q.bind(v),
                    BindArg::F64(v) => q.bind(v),
                    BindArg::Text(v) => q.bind(v),
                    BindArg::Bool(v) => q.bind(v),
                    BindArg::Uuid(v) => q.bind(v),
                    BindArg::Blob(v) => q.bind(v),
                };
                }

                let result = q.fetch_optional(executor).await;
                let elapsed = start.elapsed();

                match result {
                    Ok(row) => {
                        let exists = row.is_some();
                        debug!(exists, elapsed_us = elapsed.as_micros() as u64, "fetch_exists done");
                        Ok(exists)
                    }
                    Err(e) => {
                        debug!(error = %e, elapsed_us = elapsed.as_micros() as u64, "fetch_exists failed");
                        Err(anyhow::anyhow!(e))
                    }
                }
            }

            /// Execute DELETE query and return rows affected.
            ///
            /// Converts the current query to a DELETE statement:
            /// ```rust,ignore
            /// # async fn example(pool: sqlx::SqlitePool) -> anyhow::Result<()> {
            /// use magic_orm::prelude::*;
            ///
            /// let deleted = Observation::query()
            ///     .filter("timestamp", "<", "2025-01-01")
            ///     .delete(&pool)
            ///     .await?;
            /// # Ok(())
            /// # }
            /// ```
            pub async fn delete(self, executor: impl Executor<'_, Database = $db>) -> anyhow::Result<u64> {
                let start = Instant::now();
                let sql = self.build_delete_sql();
                debug!(sql = %sql, value_count = self.values.len(), "delete");

                let mut q = sqlx::query(&sql);
                for v in self.values {
                    q = match v {
                    BindArg::Null => q.bind(None::<i64>),
                    BindArg::I64(v) => q.bind(v),
                    BindArg::F64(v) => q.bind(v),
                    BindArg::Text(v) => q.bind(v),
                    BindArg::Bool(v) => q.bind(v),
                    BindArg::Uuid(v) => q.bind(v),
                    BindArg::Blob(v) => q.bind(v),
                };
                }

                let result = q.execute(executor).await;
                let elapsed = start.elapsed();

                match result {
                    Ok(result) => {
                        let affected = result.rows_affected();
                        debug!(affected, elapsed_us = elapsed.as_micros() as u64, "delete done");
                        Ok(affected)
                    }
                    Err(e) => {
                        debug!(error = %e, elapsed_us = elapsed.as_micros() as u64, "delete failed");
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
