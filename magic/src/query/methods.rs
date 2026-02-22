use crate::query::QueryBuilder;
use sqlx;
impl<'a, T> QueryBuilder<'a, T> 
where
    T: for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> + Send + Unpin,
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

    pub fn join(mut self, table: &str, on: &str, join_type: &str) -> Self {
        self.joins.push(format!("{} JOIN {} ON {}", join_type, table, on));
        self
    }

    pub async fn fetch_all(self, pool: &sqlx::SqlitePool) -> sqlx::Result<Vec<T>> {
        let sql = self.build_sql();

        sqlx::query_as::<_, T>(&sql)
            .fetch_all(pool)
            .await
    }

    pub async fn fetch_one(self, pool: &sqlx::SqlitePool) -> sqlx::Result<Option<T>> {
        let sql = self.build_sql();

        sqlx::query_as::<_, T>(&sql)
            .fetch_optional(pool)
            .await
    }

    fn build_sql(&self) -> String {
        let mut sql = if self.select_columns.is_empty() {
            format!("SELECT * FROM {}", self.table)
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