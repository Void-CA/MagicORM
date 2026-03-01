use crate::meta::{ColumnMeta, ForeignKeyMeta};

pub trait RegisteredModels {
    fn models() -> Vec<ModelDescriptor>;
}

pub struct ModelDescriptor {
    pub table: &'static str,
    pub columns: &'static [ColumnMeta],
    pub foreign_keys: &'static [ForeignKeyMeta],
}

#[macro_export]
macro_rules! register_models {
    ($($model:ty),* $(,)?) => {
        pub struct AppModels;

        impl $crate::registry::RegisteredModels for AppModels {
            fn models() -> Vec<$crate::registry::ModelDescriptor> {
                vec![
                    $(
                        <$model>::descriptor()
                    ),*
                ]
            }
        }
    };
}