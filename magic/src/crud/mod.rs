use std::time::Instant;

use crate::dialect::{HasDialect, SqlDialect};
use crate::model::Model;
use crate::query::statement::BindArg;
use sqlx::{Executor, Transaction};
use tracing::debug;

pub const DEFAULT_BATCH_SIZE: usize = 100;

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
                    BindArg::Null => q.bind(None::<i64>),
                    BindArg::I64(v) => q.bind(v),
                    BindArg::F64(v) => q.bind(v),
                    BindArg::Text(v) => q.bind(v),
                    BindArg::Bool(v) => q.bind(v),
                    BindArg::Uuid(v) => q.bind(v),
                    BindArg::Blob(v) => q.bind(v),
                };
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

            #[cfg(all(feature = "sqlite", not(feature = "postgres")))]
            {
                let result = q.execute(executor).await.map_err(|e| anyhow::anyhow!(e))?;
                let elapsed = start.elapsed();
                let id = result.last_insert_rowid() as i64;
                debug!(id, elapsed_us = elapsed.as_micros() as u64, "crud::insert done");
                Ok(id)
            }
        }

        // ----------------------------------------------------------------
        // insert_many — batch insert with automatic chunking
        // ----------------------------------------------------------------
        pub async fn insert_many<'e, T, I>(
            executor: impl Executor<'e, Database = $db> + Copy,
            table: &str,
            columns: &[&str],
            items: I,
        ) -> anyhow::Result<u64>
        where
            T: Model<DB = $db>,
            I: IntoIterator<Item = Vec<BindArg>>,
        {
            let start = Instant::now();
            let all_values: Vec<Vec<BindArg>> = items.into_iter().collect();
            let total = all_values.len();
            debug!(table, total, batch_size = DEFAULT_BATCH_SIZE, "crud::insert_many");

            if total == 0 {
                return Ok(0);
            }

            let cols_joined = columns.iter()
                .map(|c| <$db as HasDialect>::Dialect::quote_identifier(c))
                .collect::<Vec<_>>()
                .join(", ");

            let mut total_inserted: u64 = 0;

            for chunk in all_values.chunks(DEFAULT_BATCH_SIZE) {
                let batch_size = chunk.len();
                let placeholders_per_row = columns.len();
                let mut value_placeholders = Vec::with_capacity(batch_size);

                for _ in 0..batch_size {
                    let row_ph: Vec<String> = (0..placeholders_per_row)
                        .map(|i| <$db as HasDialect>::Dialect::placeholder(i + 1))
                        .collect();
                    value_placeholders.push(format!("({})", row_ph.join(", ")));
                }

                let sql = format!(
                    "INSERT INTO {} ({}) VALUES {}",
                    table,
                    cols_joined,
                    value_placeholders.join(", "),
                );

                let mut q = sqlx::query(&sql);
                for row_values in chunk {
                    for v in row_values {
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
                }

                {
                    let result = q.execute(executor).await.map_err(|e| anyhow::anyhow!(e))?;
                    total_inserted += result.rows_affected();
                }
            }

            let elapsed = start.elapsed();
            debug!(
                total_inserted,
                elapsed_ms = elapsed.as_secs_f64() * 1000.0,
                rows_per_sec = total_inserted as f64 / elapsed.as_secs_f64(),
                "crud::insert_many done"
            );
            Ok(total_inserted)
        }

        // ----------------------------------------------------------------
        // insert_many_in_tx — batch insert within an existing transaction
        // ----------------------------------------------------------------
        pub async fn insert_many_in_tx<'e, T, I>(
            tx: &mut Transaction<'e, $db>,
            table: &str,
            columns: &[&str],
            items: I,
        ) -> anyhow::Result<u64>
        where
            T: Model<DB = $db>,
            I: IntoIterator<Item = Vec<BindArg>>,
        {
            let start = Instant::now();
            let all_values: Vec<Vec<BindArg>> = items.into_iter().collect();
            let total = all_values.len();
            debug!(table, total, batch_size = DEFAULT_BATCH_SIZE, "crud::insert_many_in_tx");

            if total == 0 {
                return Ok(0);
            }

            let cols_joined = columns.iter()
                .map(|c| <$db as HasDialect>::Dialect::quote_identifier(c))
                .collect::<Vec<_>>()
                .join(", ");

            let mut total_inserted: u64 = 0;

            for chunk in all_values.chunks(DEFAULT_BATCH_SIZE) {
                let batch_size = chunk.len();
                let placeholders_per_row = columns.len();
                let mut value_placeholders = Vec::with_capacity(batch_size);

                for _ in 0..batch_size {
                    let row_ph: Vec<String> = (0..placeholders_per_row)
                        .map(|i| <$db as HasDialect>::Dialect::placeholder(i + 1))
                        .collect();
                    value_placeholders.push(format!("({})", row_ph.join(", ")));
                }

                let sql = format!(
                    "INSERT INTO {} ({}) VALUES {}",
                    table,
                    cols_joined,
                    value_placeholders.join(", "),
                );

                let mut q = sqlx::query(&sql);
                for row_values in chunk {
                    for v in row_values {
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
                }

                {
                    let result = q.execute(&mut **tx).await.map_err(|e| anyhow::anyhow!(e))?;
                    total_inserted += result.rows_affected();
                }
            }

            let elapsed = start.elapsed();
            debug!(
                total_inserted,
                elapsed_ms = elapsed.as_secs_f64() * 1000.0,
                rows_per_sec = total_inserted as f64 / elapsed.as_secs_f64(),
                "crud::insert_many_in_tx done"
            );
            Ok(total_inserted)
        }

        // ----------------------------------------------------------------
        // upsert (INSERT ... ON CONFLICT DO UPDATE)
        // ----------------------------------------------------------------
        pub async fn upsert<'e, T>(
            executor: impl Executor<'e, Database = $db>,
            table: &str,
            columns: &[&str],
            values: Vec<BindArg>,
        ) -> anyhow::Result<i64>
        where
            T: Model<DB = $db>,
        {
            let start = Instant::now();
            let sql = <$db as HasDialect>::Dialect::upsert_returning(table, columns, T::id_column());
            debug!(table, sql = %sql, value_count = values.len(), "crud::upsert");

            let mut q = sqlx::query(&sql);
            for v in values {
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

            #[cfg(feature = "postgres")]
            {
                use sqlx::Row;
                let row = q.fetch_one(executor).await.map_err(|e| anyhow::anyhow!(e))?;
                let elapsed = start.elapsed();
                let id: i64 = row.try_get(0).map_err(|e| anyhow::anyhow!(e))?;
                debug!(id, elapsed_us = elapsed.as_micros() as u64, "crud::upsert done");
                Ok(id)
            }

            #[cfg(all(feature = "sqlite", not(feature = "postgres")))]
            {
                let result = q.execute(executor).await.map_err(|e| anyhow::anyhow!(e))?;
                let elapsed = start.elapsed();
                let id = result.last_insert_rowid() as i64;
                debug!(id, elapsed_us = elapsed.as_micros() as u64, "crud::upsert done");
                Ok(id)
            }
        }

        // ----------------------------------------------------------------
        // upsert_with_id — upsert that includes the id column in the INSERT
        // ----------------------------------------------------------------
        pub async fn upsert_with_id<'e, T>(
            executor: impl Executor<'e, Database = $db>,
            table: &str,
            columns: &[&str],
            values: Vec<BindArg>,
        ) -> anyhow::Result<i64>
        where
            T: Model<DB = $db>,
        {
            let start = Instant::now();
            let sql = <$db as HasDialect>::Dialect::upsert_returning(table, columns, T::id_column());
            debug!(table, sql = %sql, value_count = values.len(), "crud::upsert_with_id");

            let mut q = sqlx::query(&sql);
            for v in values {
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

            #[cfg(feature = "postgres")]
            {
                use sqlx::Row;
                let row = q.fetch_one(executor).await.map_err(|e| anyhow::anyhow!(e))?;
                let elapsed = start.elapsed();
                let id: i64 = row.try_get(0).map_err(|e| anyhow::anyhow!(e))?;
                debug!(id, elapsed_us = elapsed.as_micros() as u64, "crud::upsert_with_id done");
                Ok(id)
            }

            #[cfg(all(feature = "sqlite", not(feature = "postgres")))]
            {
                let result = q.execute(executor).await.map_err(|e| anyhow::anyhow!(e))?;
                let elapsed = start.elapsed();
                let id = result.last_insert_rowid() as i64;
                debug!(id, elapsed_us = elapsed.as_micros() as u64, "crud::upsert_with_id done");
                Ok(id)
            }
        }

        // ----------------------------------------------------------------
        // upsert_many — batch upsert with automatic chunking
        // ----------------------------------------------------------------
        pub async fn upsert_many<'e, T, I>(
            executor: impl Executor<'e, Database = $db> + Copy,
            table: &str,
            columns: &[&str],
            items: I,
        ) -> anyhow::Result<u64>
        where
            T: Model<DB = $db>,
            I: IntoIterator<Item = Vec<BindArg>>,
        {
            let start = Instant::now();
            let all_values: Vec<Vec<BindArg>> = items.into_iter().collect();
            let total = all_values.len();
            debug!(table, total, batch_size = DEFAULT_BATCH_SIZE, "crud::upsert_many");

            if total == 0 {
                return Ok(0);
            }

            let cols_joined = columns.iter()
                .map(|c| <$db as HasDialect>::Dialect::quote_identifier(c))
                .collect::<Vec<_>>()
                .join(", ");

            let set_clause: Vec<String> = columns
                .iter()
                .map(|c| {
                    let q = <$db as HasDialect>::Dialect::quote_identifier(c);
                    format!("{} = excluded.{}", q, q)
                })
                .collect();
            let set_clause = set_clause.join(", ");
            let pk = T::id_column();

            let mut total_upserted: u64 = 0;

            for chunk in all_values.chunks(DEFAULT_BATCH_SIZE) {
                let batch_size = chunk.len();
                let placeholders_per_row = columns.len();
                let mut value_placeholders = Vec::with_capacity(batch_size);

                for _ in 0..batch_size {
                    let row_ph: Vec<String> = (0..placeholders_per_row)
                        .map(|i| <$db as HasDialect>::Dialect::placeholder(i + 1))
                        .collect();
                    value_placeholders.push(format!("({})", row_ph.join(", ")));
                }

                let sql = format!(
                    "INSERT INTO {} ({}) VALUES {} ON CONFLICT({}) DO UPDATE SET {}",
                    table,
                    cols_joined,
                    value_placeholders.join(", "),
                    pk,
                    set_clause,
                );

                let mut q = sqlx::query(&sql);
                for row_values in chunk {
                    for v in row_values {
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
                }

                {
                    let result = q.execute(executor).await.map_err(|e| anyhow::anyhow!(e))?;
                    total_upserted += result.rows_affected();
                }
            }

            let elapsed = start.elapsed();
            debug!(
                total_upserted,
                elapsed_ms = elapsed.as_secs_f64() * 1000.0,
                rows_per_sec = total_upserted as f64 / elapsed.as_secs_f64(),
                "crud::upsert_many done"
            );
            Ok(total_upserted)
        }

        // ----------------------------------------------------------------
        // upsert_many_in_tx — batch upsert within an existing transaction
        // ----------------------------------------------------------------
        pub async fn upsert_many_in_tx<'e, T, I>(
            tx: &mut Transaction<'e, $db>,
            table: &str,
            columns: &[&str],
            items: I,
        ) -> anyhow::Result<u64>
        where
            T: Model<DB = $db>,
            I: IntoIterator<Item = Vec<BindArg>>,
        {
            let start = Instant::now();
            let all_values: Vec<Vec<BindArg>> = items.into_iter().collect();
            let total = all_values.len();
            debug!(table, total, batch_size = DEFAULT_BATCH_SIZE, "crud::upsert_many_in_tx");

            if total == 0 {
                return Ok(0);
            }

            let cols_joined = columns.iter()
                .map(|c| <$db as HasDialect>::Dialect::quote_identifier(c))
                .collect::<Vec<_>>()
                .join(", ");

            let set_clause: Vec<String> = columns
                .iter()
                .map(|c| {
                    let q = <$db as HasDialect>::Dialect::quote_identifier(c);
                    format!("{} = excluded.{}", q, q)
                })
                .collect();
            let set_clause = set_clause.join(", ");
            let pk = T::id_column();

            let mut total_upserted: u64 = 0;

            for chunk in all_values.chunks(DEFAULT_BATCH_SIZE) {
                let batch_size = chunk.len();
                let placeholders_per_row = columns.len();
                let mut value_placeholders = Vec::with_capacity(batch_size);

                for _ in 0..batch_size {
                    let row_ph: Vec<String> = (0..placeholders_per_row)
                        .map(|i| <$db as HasDialect>::Dialect::placeholder(i + 1))
                        .collect();
                    value_placeholders.push(format!("({})", row_ph.join(", ")));
                }

                let sql = format!(
                    "INSERT INTO {} ({}) VALUES {} ON CONFLICT({}) DO UPDATE SET {}",
                    table,
                    cols_joined,
                    value_placeholders.join(", "),
                    pk,
                    set_clause,
                );

                let mut q = sqlx::query(&sql);
                for row_values in chunk {
                    for v in row_values {
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
                }

                {
                    let result = q.execute(&mut **tx).await.map_err(|e| anyhow::anyhow!(e))?;
                    total_upserted += result.rows_affected();
                }
            }

            let elapsed = start.elapsed();
            debug!(
                total_upserted,
                elapsed_ms = elapsed.as_secs_f64() * 1000.0,
                rows_per_sec = total_upserted as f64 / elapsed.as_secs_f64(),
                "crud::upsert_many_in_tx done"
            );
            Ok(total_upserted)
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
                    BindArg::Null => q.bind(None::<i64>),
                    BindArg::I64(v) => q.bind(v),
                    BindArg::F64(v) => q.bind(v),
                    BindArg::Text(v) => q.bind(v),
                    BindArg::Bool(v) => q.bind(v),
                    BindArg::Uuid(v) => q.bind(v),
                    BindArg::Blob(v) => q.bind(v),
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
#[cfg(feature = "postgres")]
impl_crud!(sqlx::Postgres);

#[cfg(all(feature = "sqlite", not(feature = "postgres")))]
impl_crud!(sqlx::Sqlite);
