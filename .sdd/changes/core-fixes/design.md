# Design: Core Fixes

## Technical Approach

Introduce generic type parameters for the database backend (`DB: sqlx::Database`) and preserve the existing generic `Id` type on the `Model` trait. Replace hardcoded `sqlx::sqlite::SqliteRow` with `sqlx::FromRow<'r, DB::Row>` bound by `DB`. Update `QueryBuilder<'a, DB, T>` and `EagerQueryBuilder<'a, DB, P, C>` to carry the DB parameter. For delegation, `EagerQueryBuilder` will implement `Deref<Target=QueryBuilder<DB, P>>` so filter/order/limit calls chain transparently. Replace the custom `executor::Executor` trait with direct use of `sqlx::Executor<'_, Database = DB>`, enabling native transaction support without wrappers.

## Architecture Decisions

### Decision 1: Generic Id Type
**Choice**: Keep `type Id` on `Model` trait, remove all `Id = i64` hardcoding in bounds.
**Alternatives considered**: Keep i64 everywhere, add generics only in relations.
**Rationale**: `Id` is already generic in the trait; the problem is downstream bounds forcing `i64`. Removing those allows UUID, String, or any `Display + Eq` type.

### Decision 2: Generic Database Backend
**Choice**: Add `DB: sqlx::Database` generic to `Model` (associated type) and propagate to all builders/loaders. Use `sqlx::FromRow<'r, DB::Row>` instead of `sqlx::FromRow<'r, sqlx::sqlite::SqliteRow>`.
**Alternatives considered**: Feature flags per DB (`#[cfg(feature = "sqlite")]`), runtime backend selection.
**Rationale**: Compile-time safety; sqlx is designed for this pattern. Feature flags would fragment code; runtime selection loses type safety.

### Decision 3: EagerQueryBuilder Delegation
**Choice**: Implement `Deref<Target=QueryBuilder<'a, DB, P>>` for `EagerQueryBuilder<'a, DB, P, C>`.
**Alternatives considered**: Macro-generated delegation, manual method forwarding, builder pattern with clone.
**Rationale**: `Deref` is zero-cost, explicit in intent, and follows Rust delegation patterns without proc-macro complexity.

### Decision 4: Transaction Support
**Choice**: Replace custom `executor::Executor` trait with `E: sqlx::Executor<'_, Database = DB>` in all query/relation functions.
**Alternatives considered**: Keep custom trait as wrapper, add `TransactionalExecutor` new type.
**Rationale**: sqlx's `Executor` already supports `Pool`, `&mut Transaction`, `&mut Connection`. Custom trait adds unnecessary indirection.

## Data Flow

```
User code:
  Model::query()           → QueryBuilder<'a, DB, Model>
       .filter(...)         → QueryBuilder<'a, DB, Model>  (delegated via Deref)
       .with_many::<Child>() → EagerQueryBuilder<'a, DB, Model, Child>
       .fetch_all(executor) → WithMany<Model, Child>
```

## File Changes

| File | Action | Description |
|------|--------|-------------|
| `magic/src/model/core.rs` | Modify | Add `type DB: sqlx::Database` to Model, update `FromRow` bounds to `DB::Row` |
| `magic/src/model/meta.rs` | No change | Already generic enough |
| `magic/src/query/builder.rs` | Modify | Add `DB: sqlx::Database` generic param, update `FromRow<'r, DB::Row>` bounds |
| `magic/src/query/executor.rs` | Modify | Add DB generic, use `sqlx::Executor<'_, Database = DB>` |
| `magic/src/query/eager/builder.rs` | Modify | Add DB generic, implement `Deref` to `QueryBuilder`, remove `Id = i64` bounds |
| `magic/src/query/eager/executor.rs` | Modify | Add DB generic, use `sqlx::Executor` |
| `magic/src/relations/traits.rs` | Modify | Relax `HasFK`: remove `P::Id: Copy` requirement |
| `magic/src/relations/loaders/has_many/eager.rs` | Modify | Generic DB backend, fix `P::Id` usage (no longer assumes i64) |
| `magic/src/executor/` | Delete/Modify | Remove custom `Executor` trait, update SQLite adapter or delete |
| `magic_derive/src/codegen/impl_model.rs` | Modify | Generate `type DB = sqlx::Sqlite;` in Model impl |
| `magic_derive/src/codegen/impl_from_row.rs` | Modify | Generate `FromRow<'r, DB::Row>` instead of hardcoded `SqliteRow` |
| `magic_derive/src/codegen/impl_has_fk.rs` | Modify | No major change needed |

## Interfaces / Contracts

```rust
// Updated Model trait
pub trait Model:
    ModelMeta
    + Sized
    + Send
    + Unpin
    + for<'r> sqlx::FromRow<'r, Self::DB::Row>
{
    type Id: Send + std::fmt::Display + Eq + Clone;
    type DB: sqlx::Database;

    fn id(&self) -> &Self::Id;
    fn query<'a>() -> QueryBuilder<'a, Self::DB, Self>;
    fn id_column() -> &'static str { "id" }
}

// Updated QueryBuilder with DB generic
pub struct QueryBuilder<'a, DB: sqlx::Database, T: Model<DB = DB>> {
    pub table: &'a str,
    pub select_columns: Vec<&'a str>,
    pub filters: Vec<String>,
    pub joins: Vec<String>,
    pub order_by: Option<String>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub _marker: PhantomData<(DB, T)>,
}

// EagerQueryBuilder with Deref delegation
pub struct EagerQueryBuilder<'a, DB: sqlx::Database, P: Model<DB = DB>, C: Model> {
    pub base: QueryBuilder<'a, DB, P>,
    pub _marker: PhantomData<C>,
}

impl<'a, DB: sqlx::Database, P: Model<DB = DB>, C: Model> Deref
    for EagerQueryBuilder<'a, DB, P, C>
{
    type Target = QueryBuilder<'a, DB, P>;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
```

## Testing Strategy

| Layer | What to Test | Approach |
|-------|-------------|----------|
| Unit | Generic traits compile | `cargo check` with models using `String` and `i64` IDs |
| Integration | Eager loading with filters | Expand `magic/tests/magic_integration.rs` with `.filter().with_many().fetch_all()` |
| Integration | Transactions | Test commit/rollback with `pool.begin().await` and `tx.commit().await` |
| Macro | Generated code compiles | Test derive macro with different ID types |

## Migration / Rollout
Breaking API change: existing models need `type DB = sqlx::Sqlite;` and may need `type Id = i64;` explicitly. No data migration needed.

## Open Questions
- [ ] Should we support non-Clone IDs? (requires careful HashMap key usage)
- [ ] Should EagerQueryBuilder use Deref or explicit method delegation?
- [ ] Should `DB` be an associated type or generic parameter on Model?

