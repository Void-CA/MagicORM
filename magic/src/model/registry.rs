/// Registra todos los modelos de la aplicación y genera la struct `AppModels`
/// que implementa [`crate::model::RegisteredModels`].
///
/// # Ejemplo
/// ```rust,ignore
/// register_models!(User, Post, Reaction);
/// // Genera: pub struct AppModels; impl RegisteredModels for AppModels { ... }
/// ```
#[macro_export]
macro_rules! register_models {
    ($($model:ty),* $(,)?) => {
        pub struct AppModels;

        impl $crate::model::RegisteredModels for AppModels {
            fn models() -> Vec<$crate::model::ModelDescriptor> {
                vec![
                    $(
                        <$model>::descriptor()
                    ),*
                ]
            }
        }
    };
}
