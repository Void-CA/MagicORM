#[macro_export]
macro_rules! register_models {
    ($($model:ty),* $(,)?) => {
        pub struct AppModels;

        impl $crate::schema::RegisteredModels for AppModels {
            fn models() -> Vec<$crate::schema::ModelDescriptor> {
                vec![
                    $(
                        <$model>::descriptor()
                    ),*
                ]
            }
        }
    };
}