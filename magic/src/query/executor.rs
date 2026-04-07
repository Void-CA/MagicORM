use crate::query::QueryBuilder;
use crate::model::ModelMeta;

use sqlx;
use sqlx::{Executor, Sqlite};

impl<'a, T> QueryBuilder<'a, T>
where
    T: ModelMeta + for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin,
{
    pub async fn fetch_all<E>(self, executor: E) -> anyhow::Result<Vec<T>>
    where
        E: Executor<'a, Database = Sqlite>,
    {
        let sql = self.build_sql();
        let rows = sqlx::query_as::<_, T>(&sql)
            .fetch_all(executor)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        Ok(rows)
    }

    pub async fn fetch_one<E>(self, executor: E) -> anyhow::Result<T>
    where
        E: Executor<'a, Database = Sqlite>,
    {
        let sql = self.build_sql();
        let row = sqlx::query_as::<_, T>(&sql)
            .fetch_one(executor)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        Ok(row)
    }

    pub async fn fetch_optional<E>(self, executor: E) -> anyhow::Result<Option<T>>
    where
        E: Executor<'a, Database = Sqlite>,
    {
        let sql = self.build_sql();
        let row = sqlx::query_as::<_, T>(&sql)
            .fetch_optional(executor)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        Ok(row)
    }

    pub async fn execute<E>(self, executor: E) -> anyhow::Result<u64>
    where
        E: Executor<'a, Database = Sqlite>,
    {
        let sql = self.build_sql();
        let result = sqlx::query(&sql)
            .execute(executor)
            .await
            .map_err(|e| anyhow::anyhow!(e))?;
        Ok(result.rows_affected())
    }
}

impl<'a, T> QueryBuilder<'a, T>
where
    T: ModelMeta
        + for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow>
        + Send
        + Unpin,
{
    pub fn new(table: &'a str) -> Self {
        Self {
            table,
            select_columns: vec![],
            filters: vec![],
            joins: vec![],
            order_by: None,
            limit: None,
            offset: None,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn select(mut self, cols: &[&'a str]) -> Self {
        self.select_columns = cols.to_vec();
        self
    }

    pub fn filter(mut self, col: &str, op: &str, value: impl ToString) -> Self {
        self.filters.push(format!("{} {} '{}'", col, op, value.to_string()));
        self
    }

    pub fn order_by(mut self, col: &str, asc: bool) -> Self {
        self.order_by = Some(format!("{} {}", col, if asc { "ASC" } else { "DESC" }));
        self
    }

    pub fn limit(mut self, lim: u32) -> Self {
        self.limit = Some(lim);
        self
    }

    pub fn offset(mut self, off: u32) -> Self {
        self.offset = Some(off);
        self
    }

    pub fn join<U>(mut self) -> Self
    where
        U: ModelMeta,
    {
        let base_table = T::TABLE;
        let join_table = U::TABLE;

        let fk = U::foreign_keys()
            .iter()
            .find(|fk| fk.related_table == base_table)
            .expect("No foreign key relationship found between models");

        let on_clause = format!(
            "{}.{} = {}.{}",
            base_table,
            fk.related_column,
            join_table,
            fk.field,
        );

        self.joins.push(format!("LEFT JOIN {} ON {}", join_table, on_clause));
        self
    }

    fn build_sql(&self) -> String {
        let mut sql = if self.select_columns.is_empty() {
            format!("SELECT * FROM {}", T::TABLE)
        } else {
            format!("SELECT {} FROM {}", self.select_columns.join(", "), self.table)
        };

        if !self.joins.is_empty() {
            sql.push(' ');
            sql.push_str(&self.joins.join(" "));
        }

        if !self.filters.is_empty() {
            sql.push_str(" WHERE ");
            sql.push_str(&self.filters.join(" AND "));
        }

        if let Some(order) = &self.order_by {
            sql.push_str(" ORDER BY ");
            sql.push_str(order);
        }

        if let Some(limit) = self.limit {
            sql.push_str(&format!(" LIMIT {}", limit));
        }

        if let Some(offset) = self.offset {
            sql.push_str(&format!(" OFFSET {}", offset));
        }

        sql
    }
}
