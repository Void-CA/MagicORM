use std::time::Instant;

use crate::dialect::{HasDialect, SqlDialect};
use crate::model::Model;
use crate::query::statement::BindArg;
use sqlx::{Database, Executor};
use tracing::debug;

// =========================================================================
// Macro — genera todas las operaciones CRUD para un DB concreto.
// =========================================================================
macro_rules! impl_crud {
    ($db:ty) => {

        // ----------------------------------------------------------------
        // insert
        // ----------------------------------------------------------------
        pub async fn insert<'e, T>(
            executor: impl Executor<'e, Database = $db>,
            table: &str,
            columns: &[&str],
            values: Vec<BindArg>,
        ) -> anyhow::Result<i64>
        where
            T: Model<DB = $db>,
        {
            let start = Instant::now();
            let sql = <$db as HasDialect>::Dialect::insert_returning(table, columns, T::id_column());
            debug!(table, sql = %sql, value_count = values.len(), "crud::insert");

            let mut q = sqlx::query(&sql);
            for v in values {
                q = match v {
                    BindArg::I64(v) => q.bind(v),
                    BindArg::F64(v) => q.bind(v),
                    BindArg::Text(v) => q.bind(v),
                    BindArg::Bool(v) => q.bind(v),
                    BindArg::Uuid(v) => q.bind(v),
                };
            }

            #[cfg(feature = "sqlite")]
            {
                let result = q.execute(executor).await.map_err(|e| anyhow::anyhow!(e))?;
                let elapsed = start.elapsed();
                let id = result.last_insert_rowid() as i64;
                debug!(id, elapsed_us = elapsed.as_micros() as u64, "crud::insert done");
                Ok(id)
            }

            #[cfg(feature = "postgres")]
            {
                use sqlx::Row;
                let row = q.fetch_one(executor).await.map_err(|e| anyhow::anyhow!(e))?;
                let elapsed = start.elapsed();
                let id: i64 = row.try_get(0).map_err(|e| anyhow::anyhow!(e))?;
                debug!(id, elapsed_us = elapsed.as_micros() as u64, "crud::insert done");
                Ok(id)
            }
        }

        // ----------------------------------------------------------------
        // get_all
        // ----------------------------------------------------------------
        pub async fn get_all<'e, T>(
            executor: impl Executor<'e, Database = $db>,
            columns: &str,
            table: &str,
        ) -> anyhow::Result<Vec<T>>
        where
            T: Model<DB = $db> + Send,
        {
            let start = Instant::now();
            let sql = format!("SELECT {} FROM {}", columns, table);
            debug!(table, sql = %sql, "crud::get_all");

            let result = sqlx::query_as::<_, T>(&sql)
                .fetch_all(executor)
                .await;
            let elapsed = start.elapsed();

            match result {
                Ok(rows) => {
                    debug!(count = rows.len(), elapsed_us = elapsed.as_micros() as u64, "crud::get_all done");
                    Ok(rows)
                }
                Err(e) => {
                    debug!(error = %e, elapsed_us = elapsed.as_micros() as u64, "crud::get_all failed");
                    Err(anyhow::anyhow!(e))
                }
            }
        }

        // ----------------------------------------------------------------
        // get_by_id
        // ----------------------------------------------------------------
        pub async fn get_by_id<'e, T>(
            executor: impl Executor<'e, Database = $db>,
            columns: &str,
            table: &str,
            id: T::Id,
        ) -> anyhow::Result<Option<T>>
        where
            T: Model<DB = $db> + Send,
        {
            let start = Instant::now();
            let placeholder = <$db as HasDialect>::Dialect::placeholder(1);
            let sql = format!("SELECT {} FROM {} WHERE id = {}", columns, table, placeholder);
            debug!(table, sql = %sql, "crud::get_by_id");

            let result = sqlx::query_as::<_, T>(&sql)
                .bind(id)
                .fetch_optional(executor)
                .await;
            let elapsed = start.elapsed();

            match result {
                Ok(row) => {
                    debug!(found = row.is_some(), elapsed_us = elapsed.as_micros() as u64, "crud::get_by_id done");
                    Ok(row)
                }
                Err(e) => {
                    debug!(error = %e, elapsed_us = elapsed.as_micros() as u64, "crud::get_by_id failed");
                    Err(anyhow::anyhow!(e))
                }
            }
        }

        // ----------------------------------------------------------------
        // put (update)
        // ----------------------------------------------------------------
        pub async fn put<'e, T>(
            executor: impl Executor<'e, Database = $db>,
            table: &str,
            columns: &[&str],
            values: Vec<BindArg>,
            id: T::Id,
        ) -> anyhow::Result<u64>
        where
            T: Model<DB = $db>,
        {
            let start = Instant::now();
            let set_clause: Vec<String> = columns
                .iter()
                .enumerate()
                .map(|(i, c)| {
                    let ph = <$db as HasDialect>::Dialect::placeholder(i + 1);
                    format!("{} = {}", c, ph)
                })
                .collect();
            let id_ph = <$db as HasDialect>::Dialect::placeholder(columns.len() + 1);
            let sql = format!(
                "UPDATE {} SET {} WHERE id = {}",
                table,
                set_clause.join(", "),
                id_ph,
            );
            debug!(table, sql = %sql, value_count = values.len(), "crud::put");

            let mut q = sqlx::query(&sql);
            for v in values {
                q = match v {
                    BindArg::I64(v) => q.bind(v),
                    BindArg::F64(v) => q.bind(v),
                    BindArg::Text(v) => q.bind(v),
                    BindArg::Bool(v) => q.bind(v),
                    BindArg::Uuid(v) => q.bind(v),
                };
            }
            q = q.bind(id);

            let result = q.execute(executor).await;
            let elapsed = start.elapsed();

            match result {
                Ok(result) => {
                    let affected = result.rows_affected();
                    debug!(affected, elapsed_us = elapsed.as_micros() as u64, "crud::put done");
                    Ok(affected)
                }
                Err(e) => {
                    debug!(error = %e, elapsed_us = elapsed.as_micros() as u64, "crud::put failed");
                    Err(anyhow::anyhow!(e))
                }
            }
        }

        // ----------------------------------------------------------------
        // delete_all
        // ----------------------------------------------------------------
        pub async fn delete_all<'e>(
            executor: impl Executor<'e, Database = $db>,
            table: &str,
        ) -> anyhow::Result<u64> {
            let start = Instant::now();
            let sql = format!("DELETE FROM {}", table);
            debug!(table, sql = %sql, "crud::delete_all");

            let result = sqlx::query(&sql)
                .execute(executor)
                .await;
            let elapsed = start.elapsed();

            match result {
                Ok(result) => {
                    let affected = result.rows_affected();
                    debug!(affected, elapsed_us = elapsed.as_micros() as u64, "crud::delete_all done");
                    Ok(affected)
                }
                Err(e) => {
                    debug!(error = %e, elapsed_us = elapsed.as_micros() as u64, "crud::delete_all failed");
                    Err(anyhow::anyhow!(e))
                }
            }
        }

        // ----------------------------------------------------------------
        // delete_by_id
        // ----------------------------------------------------------------
        pub async fn delete_by_id<'e, T>(
            executor: impl Executor<'e, Database = $db>,
            table: &str,
            id: T::Id,
        ) -> anyhow::Result<u64>
        where
            T: Model<DB = $db>,
        {
            let start = Instant::now();
            let placeholder = <$db as HasDialect>::Dialect::placeholder(1);
            let sql = format!("DELETE FROM {} WHERE id = {}", table, placeholder);
            debug!(table, sql = %sql, "crud::delete_by_id");

            let result = sqlx::query(&sql)
                .bind(id)
                .execute(executor)
                .await;
            let elapsed = start.elapsed();

            match result {
                Ok(result) => {
                    let affected = result.rows_affected();
                    debug!(affected, elapsed_us = elapsed.as_micros() as u64, "crud::delete_by_id done");
                    Ok(affected)
                }
                Err(e) => {
                    debug!(error = %e, elapsed_us = elapsed.as_micros() as u64, "crud::delete_by_id failed");
                    Err(anyhow::anyhow!(e))
                }
            }
        }
    };
}

// =========================================================================
// Implementaciones concretas
// =========================================================================
#[cfg(feature = "sqlite")]
impl_crud!(sqlx::Sqlite);

#[cfg(feature = "postgres")]
impl_crud!(sqlx::Postgres);
