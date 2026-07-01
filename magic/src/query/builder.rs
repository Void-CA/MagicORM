use std::marker::PhantomData;
use crate::dialect::{HasDialect, SqlDialect};
use crate::model::{ModelMeta, Model};
use crate::query::statement::BindArg;

// ---------------------------------------------------------------------------
// Filter — condición sin formatear; el placeholder se genera al buildear
// ---------------------------------------------------------------------------
struct Filter {
    column: String,
    operator: String,
}

// ---------------------------------------------------------------------------
// QueryBuilder — constructor de consultas SQL parametrizadas.
//
// Separa la construcción (build_sql) de la ejecución (fetch_all etc).
// Los valores se almacenan como BindArg y se bindean al ejecutar,
// eliminando por completo la posibilidad de SQL injection.
// ---------------------------------------------------------------------------
pub struct QueryBuilder<'a, DB: sqlx::Database, T: ModelMeta> {
    pub table: &'a str,
    pub select_columns: Vec<String>,
    filters: Vec<Filter>,
    pub joins: Vec<String>,
    pub order_by: Option<(String, bool)>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    /// Valores para bindear, en el mismo orden que aparecen en el SQL final.
    pub values: Vec<BindArg>,
    pub _marker: PhantomData<(DB, T)>,
}

impl<'a, DB: sqlx::Database + HasDialect, T: ModelMeta> QueryBuilder<'a, DB, T> {
    pub fn new(table: &'a str) -> Self {
        Self {
            table,
            select_columns: vec![],
            filters: vec![],
            joins: vec![],
            order_by: None,
            limit: None,
            offset: None,
            values: vec![],
            _marker: PhantomData,
        }
    }

    pub fn select(mut self, cols: &[&str]) -> Self {
        self.select_columns = cols.iter().map(|c| c.to_string()).collect();
        self
    }

    /// Agrega un filtro con binding parametrizado.
    /// El valor NO se interpolan en el SQL — se bindea como parámetro.
    pub fn filter(mut self, col: &str, op: &str, value: impl Into<BindArg>) -> Self {
        self.filters.push(Filter {
            column: col.to_string(),
            operator: op.to_string(),
        });
        self.values.push(value.into());
        self
    }

    pub fn order_by(mut self, col: &str, asc: bool) -> Self {
        self.order_by = Some((col.to_string(), asc));
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

        let fks = U::foreign_keys();
        let fk = fks
            .iter()
            .find(|fk| fk.related_table == base_table)
            .expect("No foreign key relationship found between models");

        let on_clause = format!(
            "{}.{} = {}.{}",
            base_table, fk.related_column, join_table, fk.field,
        );

        self.joins.push(format!("LEFT JOIN {} ON {}", join_table, on_clause));
        self
    }

    // ------------------------------------------------------------------
    // build_sql — genera el SQL con placeholders del dialecto correcto
    // ------------------------------------------------------------------
    pub fn build_sql(&self) -> String {
        let mut placeholder_idx = 0;

        let mut sql = if self.select_columns.is_empty() {
            format!("SELECT * FROM {}", T::TABLE)
        } else {
            let cols = self.select_columns.join(", ");
            format!("SELECT {} FROM {}", cols, self.table)
        };

        if !self.joins.is_empty() {
            sql.push(' ');
            sql.push_str(&self.joins.join(" "));
        }

        if !self.filters.is_empty() {
            sql.push_str(" WHERE ");
            for (i, f) in self.filters.iter().enumerate() {
                if i > 0 {
                    sql.push_str(" AND ");
                }
                placeholder_idx += 1;
                let ph = DB::Dialect::placeholder(placeholder_idx);
                sql.push_str(&format!("{} {} {}", f.column, f.operator, ph));
            }
        }

        if let Some((col, asc)) = &self.order_by {
            let dir = if *asc { "ASC" } else { "DESC" };
            sql.push_str(&format!(" ORDER BY {} {}", col, dir));
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

// ------------------------------------------------------------------
// with_many — solo disponible cuando T: Model (no solo ModelMeta)
// ------------------------------------------------------------------
impl<'a, DB, T> QueryBuilder<'a, DB, T>
where
    DB: sqlx::Database + HasDialect,
    T: Model<DB = DB> + ModelMeta,
    T::Id: Clone + Eq + std::hash::Hash + std::fmt::Display
        + for<'q> sqlx::Encode<'q, DB> + sqlx::Type<DB>,
{
    pub fn with_many<C>(self) -> crate::query::EagerQueryBuilder<'a, DB, T, C>
    where
        C: Model<DB = DB> + ModelMeta + crate::relations::traits::HasFK<T> + Send + Unpin,
    {
        crate::query::EagerQueryBuilder {
            _marker: PhantomData,
            inner: self,
        }
    }
}
