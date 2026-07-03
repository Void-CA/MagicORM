use crate::model::{Model, ModelMeta};
use crate::prelude::HasFK;
use crate::query::eager::{EagerQueryBuilder, WithMany};

use sqlx::Executor;

macro_rules! impl_eager_executor {
    ($db:ty) => {
        impl<'a, P, C> EagerQueryBuilder<'a, $db, P, C>
        where
            P: Model<DB = $db> + ModelMeta + Send + Unpin,
            P::Id: Clone + Eq + std::hash::Hash,
            C: Model<DB = $db> + ModelMeta + HasFK<P> + Send + Unpin,
        {
            /// Ejecuta eager loading.
            /// Nota: requiere executor Copy (&pool, no &mut tx).
            /// Para usar dentro de una transacción, pasa los datos manualmente.
            pub async fn fetch_all(self, executor: impl Executor<'_, Database = $db> + Copy) -> anyhow::Result<WithMany<P, C>> {
                let parents = self.inner.fetch_all(executor).await?;

                let children =
                    crate::relations::loaders::has_many::eager::load_has_many_batch::<P, C>(
                        &parents,
                        executor,
                    )
                    .await?;

                Ok(WithMany { parents, children })
            }
        }
    };
}

#[cfg(feature = "sqlite")]
impl_eager_executor!(sqlx::Sqlite);

#[cfg(feature = "postgres")]
impl_eager_executor!(sqlx::Postgres);
