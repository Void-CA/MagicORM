#[macro_export]
macro_rules! has_many {
    ($model:ident => $($child:ident),+ $(,)?) => {
        use paste::paste;
        paste! {
            // 1. Crear struct de relaciones
            pub struct [<$model Relations>];

            impl $crate::relations::traits::RelationList for [<$model Relations>] {
                fn all() -> Vec<&'static str> {
                    vec![$(stringify!($child)),+]
                }
            }

            impl $crate::relations::traits::HasRelations for $model {
                type HasMany = [<$model Relations>];
            }

            // 2. Métodos de lazy loading
            impl $model {
                $(
                    pub async fn $child(&self, pool: &sqlx::SqlitePool) -> anyhow::Result<Vec<$child>> {
                        $crate::relations::runtime::has_many::load_has_many::<$model, $child>(pool, self.id, self.id_column()).await
                    }
                )+
            }
        }
    };
}