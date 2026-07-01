use std::marker::PhantomData;
use crate::dialect::{HasDialect, SqlDialect};
use crate::model::{ModelMeta, Model};
use crate::query::statement::BindArg;

// ---------------------------------------------------------------------------
// Filter — condición sin formatear
// ---------------------------------------------------------------------------
struct Filter {
    column: String,
    operator: String,
    /// true si usa OR en vez de AND para conectar
    is_or: bool,
    /// cantidad de placeholders (1 para filtros normales, >1 para IN)
    value_count: usize,
}

// ---------------------------------------------------------------------------
// QueryBuilder
// ---------------------------------------------------------------------------
pub struct QueryBuilder<'a, DB: sqlx::Database, T: ModelMeta> {
    pub table: &'a str,
    pub select_columns: Vec<String>,
    filters: Vec<Filter>,
    pub joins: Vec<String>,
    pub order_by: Option<(String, bool)>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub values: Vec<BindArg>,
    /// Modo count: SELECT count(*) en vez de SELECT columnas
    count_mode: bool,
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
            count_mode: false,
            _marker: PhantomData,
        }
    }

    pub fn select(mut self, cols: &[&str]) -> Self {
        self.select_columns = cols.iter().map(|c| c.to_string()).collect();
        self
    }

    /// Filtro AND: col op ?
    pub fn filter(mut self, col: &str, op: &str, value: impl Into<BindArg>) -> Self {
        self.filters.push(Filter {
            column: col.to_string(),
            operator: op.to_string(),
            is_or: false,
            value_count: 1,
        });
        self.values.push(value.into());
        self
    }

    /// Filtro OR: agrega OR col op ? en vez de AND
    pub fn or_filter(mut self, col: &str, op: &str, value: impl Into<BindArg>) -> Self {
        self.filters.push(Filter {
            column: col.to_string(),
            operator: op.to_string(),
            is_or: true,
            value_count: 1,
        });
        self.values.push(value.into());
        self
    }

    /// Filtro IN: col IN (?, ?, ...)
    pub fn filter_in(mut self, col: &str, values: impl IntoIterator<Item = impl Into<BindArg>>) -> Self {
        let vals: Vec<BindArg> = values.into_iter().map(|v| v.into()).collect();
        let count = vals.len();
        if count == 0 {
            return self; // IN vacío no agrega filtro
        }
        self.filters.push(Filter {
            column: col.to_string(),
            operator: "IN".to_string(),
            is_or: false,
            value_count: count,
        });
        self.values.extend(vals);
        self
    }

    /// Modo COUNT: SELECT count(*) FROM ...
    pub fn count(mut self) -> Self {
        self.count_mode = true;
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
    // build_sql
    // ------------------------------------------------------------------
    pub fn build_sql(&self) -> String {
        let mut placeholder_idx = 0;

        let mut sql = if self.count_mode {
            format!("SELECT count(*) FROM {}", T::TABLE)
        } else if self.select_columns.is_empty() {
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
                    sql.push_str(if f.is_or { " OR " } else { " AND " });
                }

                if f.operator == "IN" {
                    // Genera: col IN (?, ?, ...)
                    let phs: Vec<String> = (1..=f.value_count)
                        .map(|_| {
                            placeholder_idx += 1;
                            DB::Dialect::placeholder(placeholder_idx)
                        })
                        .collect();
                    sql.push_str(&format!("{} IN ({})", f.column, phs.join(", ")));
                } else {
                    placeholder_idx += 1;
                    let ph = DB::Dialect::placeholder(placeholder_idx);
                    sql.push_str(&format!("{} {} {}", f.column, f.operator, ph));
                }
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

    /// Helper: aplica limit(1) + order_by(id DESC) para first()
    pub fn first_query(mut self) -> Self {
        self.limit = Some(1);
        if self.order_by.is_none() {
            self.order_by = Some((T::TABLE.to_string() + ".id", false));
        }
        self
    }
}

// ------------------------------------------------------------------
// with_many
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
