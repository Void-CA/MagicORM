pub struct QueryBuilder<'a, T> {
    pub table: &'a str,
    pub select_columns: Vec<&'a str>,
    pub filters: Vec<String>,
    pub order_by: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub _marker: std::marker::PhantomData<T>,
}