#[macro_export]
macro_rules! has_many {
    ($model:ident => $($child:ident),+ $(,)?) => {
        ::paste::paste! {
            // 1. Crear struct de relaciones (solo si quieres introspección)
            pub struct [<$model Relations>];

            impl $crate::relations::traits::RelationList for [<$model Relations>] {
                fn all() -> Vec<&'static str> {
                    vec![$(stringify!($child)),+]
                }
            }

            impl $crate::relations::traits::HasRelations for $model {
                type HasMany = [<$model Relations>];
            }

            // 2. Métodos de lazy loading (snake_case, plural simple)
            impl $model {
                $(
                    pub async fn [<$child:snake s>]<'e, E>(&self, executor: E) -> anyhow::Result<Vec<$child>>
                    where
                        E: sqlx::Executor<'e, Database = sqlx::Sqlite>,
                    {
                        $crate::relations::load_has_many::<$model, $child, E>(self, executor).await
                    }
                )+
            }
        }
    };
}