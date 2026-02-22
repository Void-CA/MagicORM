use crate::query::QueryBuilder;
use sqlx;
impl<'a, T> QueryBuilder<'a, T> {
    pub fn new(table: &'a str) -> Self {
        Self {
            table,
            select_columns: vec![],
            filters: vec![],
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

    
}