pub trait Model: Sized + Send {
    type Id: Send;

    fn table_name() -> &'static str;
    fn columns() -> &'static [&'static str];
    fn id_column() -> &'static str {
        "id"
    }
}

pub trait BelongsTo<P: Model>: Model {
    fn foreign_key() -> &'static str;
}

pub trait HasMany<C>: Model
where
    C: BelongsTo<Self>,
{
}