use crate::model::{Model, ModelMeta};
use crate::prelude::HasFK;
use crate::query::eager::{EagerQueryBuilder, WithMany};

impl<'a, P, C> EagerQueryBuilder<'a, P, C>
where
    P: Model<Id = i64> + ModelMeta + Send + Unpin,
    C: Model
        + ModelMeta
        + HasFK<P>
        + Send
        + Unpin
        + for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow>,
{
    pub async fn fetch_all<E>(self, executor: E) -> anyhow::Result<WithMany<P, C>>
    where
        E: sqlx::Executor<'a, Database = sqlx::Sqlite> + Copy,
    {
        // 1️⃣ traer padres
        let parents = self.base.fetch_all(executor).await?;

        // 2️⃣ eager loading directo
        let children =
            crate::relations::loaders::has_many::eager::load_has_many_batch::<P, C, E>(
                &parents,
                executor,
            )
            .await?;

        Ok(WithMany { parents, children })
    }
}