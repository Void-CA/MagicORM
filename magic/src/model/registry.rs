/// Registra todos los modelos de la aplicación y genera la struct `AppModels`
/// que implementa [`crate::model::RegisteredModels`] y describe todos los modelos.
///
/// # Ejemplo
/// ```rust,ignore
/// register_models!(User, Post, Reaction);
/// // Genera:
/// //   pub struct AppModels;
/// //   impl RegisteredModels for AppModels { ... }
/// //   fn all_descriptors() -> Vec<ModelDescriptor> { ... }
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

        /// Retorna los descriptors de todos los modelos registrados.
        /// Útil para CLI, migraciones, introspección, etc.
        pub fn all_descriptors() -> Vec<$crate::model::ModelDescriptor> {
            vec![
                $(
                    <$model as $crate::describe::Describe>::descriptor()
                ),*
            ]
        }
    };
}
