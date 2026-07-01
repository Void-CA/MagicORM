# MagicORM Architecture Plan

## Decisiones firmes (Jun 2026)

### Visión de capas

```
magic_derive (proc macros)
    → genera solo wiring, no conoce SQL
    → NewStruct + impl Model + ModelMeta + HasFK + BelongsTo
    → CRUD delega en helpers compartidos de magic_orm

magic_orm (core library)
    ├── dialect/     SqlDialect trait + impls (Sqlite, Postgres, MySQL)
    ├── query/       QueryBuilder<DB, T> → build() → Statement<DB>
    ├── executor/    helpers sobre sqlx::Executor (sin custom Executor trait)
    ├── crud/        insert/read/update/delete compartidos, multi-DB
    ├── relations/   HasMany / BelongsTo loaders genéricos sobre DB
    └── schema/      create_all + diff (para migraciones, en Fase B+)

magic_cli
    → magic db init
    → magic migrate generate / up / down / status (Fase B+)
```

### Principios

1. **El proc macro NO conoce SQL.** Genera metadatos y delega ejecución.
2. **El core NO conoce SQLite.** Conoce `DB: sqlx::Database` y `D: SqlDialect`.
3. **El CLI NO conoce modelos Rust en runtime.** Consume descriptors serializados.
4. **Separación estricta:** Construcción → Compilación SQL → Ejecución.
5. **Cero side effects de filesystem en proc macros.** No más `write_model_json()`.

### Decisiones lockeadas (Fase A)

| Decisión | Opción elegida | Alternativa descartada |
|----------|---------------|----------------------|
| Database backend | `type DB: sqlx::Database` associated type en Model | Parámetro genérico, feature flags |
| SQL dialecto | `SqlDialect` trait + `HasDialect<DB>` | Feature flags por DB |
| Query/Execute split | `build() -> Statement<DB>`, ejecución aparte | QueryBuilder que ejecuta directo |
| EagerQueryBuilder | `Deref<Target=QueryBuilder<DB, P>>` | Delegación manual |
| CRUD | Derive genera delegación a `magic_orm::crud::*` | Derive genera SQL inline |
| Executor legacy | Borrar el trait custom | Mantener wrapper |
| DefaultDB | Feature-flag driven, `sqlite` es default | Sin default |
| Descriptors exposure | **POSTERGADO a Fase B.** Candidato: derive genera `impl Describe` + `register_models!`, CLI ejecuta binario del proyecto a stdout | build.rs, FFI, proc macro side effects |

### Roadmap

#### Fase A (ahora)
1. `SqlDialect` trait + `SqliteDialect` + `PostgresDialect`
2. `Statement<DB>` (SQL + bindings tipados)
3. `QueryBuilder<'a, DB, T>` con `build() -> Statement<DB>`, sin SQL injection
4. `Model::DB` associated type, `FromRow` sobre `DB::Row`
5. CRUD compartido en `magic_orm::crud::*`, derive genera solo delegación
6. Borrar `Executor` trait legacy
7. `has_many!` macro genera firma con `T::DB`
8. Tests: SQL injection, dialect snapshot, generic ID

**Criterio de salida:**
- `cargo check --no-default-features --features postgres` compila
- `cargo check` (default sqlite) compila
- Tests existentes pasan
- Tests nuevos pasan
- Un modelo con `id: String` funciona end-to-end con SQLite

#### Fase B (posterior)
1. CRUD compartido completo (multi-backend)
2. Estabilizar `ModelMeta` / `ColumnMeta` / `ForeignKeyMeta` / `ModelDescriptor`
3. Tests de SQL generado con `insta`
4. Logging/tracing
5. Mecanismo de exposure de descriptors (cuarta opción: binary stdout)
6. Migraciones end-to-end

#### Fase C (post-freeze API)
- API freeze, DSL tipado opcional, paginación, carga anidada

#### Fase D
- Soft delete, timestamps, hooks, validaciones

### Contratos clave

```rust
// SqlDialect — abstrae diferencias entre backends SQL
pub trait SqlDialect: Send + Sync + 'static {
    fn placeholder(index: usize) -> String;       // "?" o "$1"
    fn quote_identifier(name: &str) -> String;    // "\"name\"" o "`name`"
    fn insert_returning(table: &str, cols: &[&str], pk: &str) -> String;
    fn last_insert_id_expr() -> Option<&'static str>;  // None si usa RETURNING
    fn enable_foreign_keys() -> Option<&'static str>;
    fn map_rust_type(rust_ty: &str) -> &'static str;
}

// Statement — resultado de build(), sin ejecución
pub struct Statement<DB: sqlx::Database> {
    pub sql: String,
    pub values: Vec<BindArg>,
}

// Model con DB asociada
pub trait Model:
    ModelMeta + Sized + Send + Unpin
    + for<'r> sqlx::FromRow<'r, <Self::DB as sqlx::Database>::Row>
{
    type Id: Send + Display + Clone + Eq + Hash
        + for<'q> sqlx::Encode<'q, Self::DB> + sqlx::Type<Self::DB>;
    type DB: sqlx::Database;
    fn id(&self) -> &Self::Id;
    fn query<'a>() -> QueryBuilder<'a, Self::DB, Self>;
    fn id_column() -> &'static str { "id" }
}
```

### BindArg (type-erased query parameter)

```rust
pub enum BindArg {
    I64(i64),
    F64(f64),
    Text(String),
    Bool(bool),
    // Future: Bytes, Uuid, Custom
}
```

Cada variante sabe bindearse a un `sqlx::Query`. Esto elimina SQL injection y permite parameterized queries sin closures ni trait objects complejos.
