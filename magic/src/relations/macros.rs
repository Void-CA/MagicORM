#[macro_export]
macro_rules! has_many {
    ($model:ident => $($child:ident),+ $(,)?) => {
        use paste::paste;
        paste! {
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
                    pub async fn [<$child:snake s>](&self, pool: &sqlx::SqlitePool) -> anyhow::Result<Vec<$child>> {
                        $crate::relations::runtime::has_many::load_has_many::<$model, $child>(self, pool).await
                    }
                )+
            }
        }
    };
}