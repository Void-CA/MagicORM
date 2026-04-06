Actúa como un Arquitecto de Software experto en Rust.

Necesito realizar un refactor estructural de mi proyecto magic_orm. El objetivo es eliminar la fragmentación del concepto "Model", separar claramente el código de tiempo de compilación del de tiempo de ejecución, y preparar el crate core para ser una librería pura.
1. Contexto de la Estructura Actual:

~/projects/rust/magic_orm  main  ls                                                                                                                                               
Cargo.lock  Cargo.toml  magic  magic_cli  magic_derive  README.md  target  test.db  TODO.md

tree magic/src                                                                                                                                   
magic/src
├── executor
│   ├── error.rs
│   └── mod.rs
├── lib.rs
├── main.rs
├── meta
│   ├── column.rs
│   ├── foreign_key.rs
│   ├── model.rs
│   └── mod.rs
├── prelude.rs
├── query
│   ├── methods.rs
│   ├── model.rs
│   └── mod.rs
├── registry.rs
├── relations
│   ├── macros.rs
│   ├── mod.rs
│   ├── runtime
│   │   ├── belongs_to.rs
│   │   ├── has_many.rs
│   │   └── mod.rs
│   └── traits.rs
├── schema
│   ├── create.rs
│   ├── models.rs
│   ├── mod.rs
│   └── utils.rs
└── traits
    ├── model.rs
    └── mod.rs

8 directories, 25 files

tree magic_derive/src                                                                                                                            
magic_derive/src
├── attrs
│   ├── foreign_key.rs
│   ├── magic.rs
│   └── mod.rs
├── expand.rs
├── generators
│   ├── files.rs
│   ├── implementations.rs
│   ├── methods.rs
│   ├── mod.rs
│   └── utils
│       ├── model_meta.rs
│       └── mod.rs
├── lib.rs
├── model.rs
└── operations
    ├── crud
    │   ├── delete.rs
    │   ├── get.rs
    │   ├── insert.rs
    │   ├── mod.rs
    │   └── put.rs
    └── mod.rs

6 directories, 18 files 

2. Problemas Detectados:

    Fragmentación: El concepto "Model" existe en meta/, query/, schema/ y traits/. Necesito centralizar la definición del modelo.

    Mezcla de responsabilidades: magic/src/main.rs debe desaparecer para que el crate sea una lib pura.

    Ambigüedad en relaciones: relations/ mezcla macros, traits y lógica de runtime.

    Escalabilidad en Executor: Actualmente es muy simple y no contempla adaptadores o pools de conexión futuros.

3. Instrucciones de Refactor:

Por favor, genera una propuesta de nueva estructura de archivos y los pasos para mover el código siguiendo estos lineamientos:

    Crate magic:

        Crea un módulo unificado model/ que absorba meta/, registry.rs y los traits base del modelo.

        Mueve la lógica de main.rs a un directorio examples/.

        Estructura executor/ para soportar múltiples backends (ej. executor/adapters/).

        Limpia relations/ separando claramente la API pública de los helpers internos.

    Crate magic_derive:

        Renombra y organiza generators/ para que los nombres de los archivos reflejen qué implementación de Rust están generando (ej. query_impl.rs en lugar de methods.rs).

        Clarifica la relación entre expand.rs y model.rs.

4. Resultado esperado:

    Un nuevo tree de directorios propuesto.

    Una tabla de equivalencias: "Archivo Viejo -> Archivo Nuevo".

    Breve explicación de cómo manejar los cambios en los mod.rs para no romper la visibilidad actual.