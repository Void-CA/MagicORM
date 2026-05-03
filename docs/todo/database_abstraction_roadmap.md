# Roadmap: Database Abstraction for magic_orm

## Current State (May 2026)

### ✅ What's Working
- `Model::Id` is generic with `Clone + Eq + Hash + Display` bounds
- `QueryBuilder` is database-agnostic (no DB-specific code)
- Basic CRUD operations work with SQLite

### ❌ Critical SQLite Coupling

#### 1. **Hardcoded SQLite Types in Trait Bounds**
```rust
// Everywhere in the codebase:
T: for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow>  // ❌
E: sqlx::Executor<'a, Database = Sqlite>                    // ❌
T::Id: sqlx::Encode<'q, sqlx::Sqlite>                     // ❌
```

**Files affected:**
- `magic/src/model/core.rs` (Model trait)
- `magic/src/query/executor.rs` (QueryBuilder bounds)
- `magic/src/query/eager/executor.rs` (EagerQueryBuilder bounds)
- `magic/src/relations/loaders/*/` (all loaders)
- `magic/src/relations/traits.rs` (HasFK trait)

#### 2. **SQLite-Specific Logic**
- `last_insert_rowid()` in `magic_derive/src/operations/crud/insert.rs`
- `PRAGMA foreign_keys = ON` in examples/tests
- `map_rust_to_sqlite()` in `magic_derive/src/codegen/utils/type_mapping.rs`

#### 3. **Exposed SQLite Types in Prelude**
```rust
// magic/src/prelude.rs
pub use sqlx::SqlitePool;  // ❌ Should be generic
```

#### 4. **Single Executor Adapter**
- Only `magic/src/executor/adapters/sqlite.rs` exists
- Assumes `SqlitePool` and `SqliteConnection`

---

## Roadmap to Database Abstraction

### Phase 1: Generic Database Trait (Foundation)
**Goal**: Remove hardcoded `SqliteRow` and `Sqlite` from core traits.

#### 1.1 Add Associated Database Type to Model
```rust
// magic/src/model/core.rs
pub trait Model: ModelMeta + Sized + Send + Unpin {
    type Id: Send + Display + Clone + Eq + Hash;
    type DB: sqlx::Database;  // NEW: Associated database type
    
    fn id(&self) -> &Self::Id;
    // ...
}
```

#### 1.2 Update FromRow Bounds
```rust
// Before:
T: for<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow>

// After:
T: for<'r> sqlx::FromRow<'r, sqlx::SqliteRow> + Send + Unpin,
where T::DB: sqlx::Database
// Or better: use generic row type
```

#### 1.3 Create DatabaseExecutor Trait
```rust
// magic/src/executor/traits.rs (new or updated)
pub trait DatabaseExecutor<'e>: sqlx::Executor<'e, Database = Self::DB> {
    type DB: sqlx::Database;
    // Add common methods
}
```

---

### Phase 2: Generic Query Bounds
**Goal**: Make QueryBuilder and loaders work with any sqlx::Database.

#### 2.1 Update QueryBuilder Bounds
```rust
// magic/src/query/executor.rs
impl<'a, T> QueryBuilder<'a, T>
where
    T: Model + ModelMeta + Send + Unpin,
    T::DB: sqlx::Database,
    // Remove hardcoded SqliteRow, use generic:
    T: for<'r> sqlx::FromRow<'r, <T::DB as sqlx::Database>::Row>,
    T::Id: Clone + Eq + Hash + Display + for<'q> sqlx::Encode<'q, T::DB> + sqlx::Type<T::DB>,
{
    // ...
}
```

#### 2.2 Update All Loaders
- `magic/src/relations/loaders/has_many/eager.rs`
- `magic/src/relations/loaders/has_many/lazy.rs`
- `magic/src/relations/loaders/belongs_to/lazy.rs`
- `magic/src/query/eager/builder.rs`
- `magic/src/query/eager/executor.rs`

All need the same generic treatment.

---

### Phase 3: SQL Dialect Abstraction
**Goal**: Handle SQL syntax differences (parameter placeholders, functions).

#### 3.1 Create SqlGenerator Trait
```rust
pub trait SqlGenerator {
    fn placeholder(index: usize) -> String;  // SQLite: "?", Postgres: "$1"
    fn last_insert_id_sql() -> &'static str;
    fn enable_foreign_keys_sql() -> &'static str;
}
```

#### 3.2 Implement for Each DB
```rust
pub struct SqliteSqlGenerator;
impl SqlGenerator for SqliteSqlGenerator {
    fn placeholder(index: usize) -> String { "?".to_string() }
    fn last_insert_id_sql() -> &'static str { "SELECT last_insert_rowid()" }
}

pub struct PostgresSqlGenerator;
impl SqlGenerator for PostgresSqlGenerator {
    fn placeholder(index: usize) -> String { format!("${}", index) }
    fn last_insert_id_sql() -> &'static str { "RETURNING id" }
}
```

#### 3.3 Update Type Mapping
```rust
// magic_derive/src/codegen/utils/type_mapping.rs
pub fn map_rust_to_sql(ty: &syn::Type, db_type: &str) -> &'static str {
    match db_type {
        "sqlite" => map_rust_to_sqlite(ty),
        "postgres" => map_rust_to_postgres(ty),
        _ => "TEXT",
    }
}
```

---

### Phase 4: Executor Adapters
**Goal**: Support multiple connection pool types.

#### 4.1 Update Existing Adapter
```rust
// magic/src/executor/adapters/sqlite.rs
impl Executor for SqlitePool { /* ... */ }
```

#### 4.2 Add Postgres Adapter
```rust
// magic/src/executor/adapters/postgres.rs (new)
use sqlx::postgres::PgPool;

impl Executor for PgPool {
    // Implement required methods
}
```

#### 4.3 Generic Pool Type
```rust
// magic/src/prelude.rs
// Before: pub use sqlx::SqlitePool;
// After:
#[cfg(feature = "sqlite")]
pub use sqlx::SqlitePool;

#[cfg(feature = "postgres")]
pub use sqlx::postgres::PgPool;
```

---

### Phase 5: Feature Flags
**Goal**: Allow users to choose database at compile time.

#### 5.1 Update Cargo.toml
```toml
[features]
default = ["sqlite"]
sqlite = ["sqlx/sqlite"]
postgres = ["sqlx/postgres"]
```

#### 5.2 Conditional Compilation
```rust
#[cfg(feature = "sqlite")]
pub type DefaultPool = SqlitePool;

#[cfg(feature = "postgres")]
pub type DefaultPool = PgPool;
```

---

## Implementation Order

1. **Phase 1** (1-2 days): Add `type DB` to Model, update core traits
2. **Phase 2** (2-3 days): Update all bounds in queries and loaders
3. **Phase 3** (3-4 days): SQL dialect abstraction
4. **Phase 4** (2-3 days): Executor adapters
5. **Phase 5** (1 day): Feature flags

**Total estimated time**: 9-13 days for complete abstraction.

---

## Risks & Considerations

1. **sqlx::Database is a complex trait** - may require nightly features
2. **Associated type defaults** - `type DB = sqlx::Sqlite;` as default to maintain backward compatibility
3. **Breaking changes** - this will break existing user code that relies on hardcoded SQLite types
4. **Testing** - need integration tests for both SQLite and Postgres

---

## Next Step

Start with **Phase 1.1**: Add `type DB` to Model trait and update the derive macro to support it.
