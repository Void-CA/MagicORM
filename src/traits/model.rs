use async_trait::async_trait;
use sqlx::Error;

#[async_trait]
pub trait Model: Sized + Send {
    type Id: Send;

    fn table_name() -> &'static str;
    fn id_column() -> &'static str {
        "id"
    }

    async fn find(id: Self::Id) -> Result<Self, Error>;
    async fn all() -> Result<Vec<Self>, Error>;
    async fn insert(&mut self) -> Result<(), Error>;
    async fn update(&self) -> Result<(), Error>;
    async fn delete(&self) -> Result<(), Error>;
}