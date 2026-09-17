# Decision: Transaction API

## Status

Proposed — 2026-09-08

## Context

MagicORM now has explicit Connection and Pool abstractions. Transactions are used in Thalos for atomic operations like:

```rust
ExecutionSession + initial state + config snapshot + execution event
```

Currently, users work with sqlx transactions directly:

```rust
let mut tx = pool.begin().await?;
User::insert(&mut *tx, &new_user).await?;
tx.commit().await?;
```

This works but raises the question: should MagicORM wrap this?

## Options

### Option A: Expose sqlx transactions directly (current)

```rust
let mut tx = pool.begin().await?;
Model::insert(&mut *tx, &data).await?;
tx.commit().await?;
```

**Pros:**
- Zero abstraction overhead
- Users familiar with sqlx feel at home
- No MagicORM-specific learning curve

**Cons:**
- No MagicORM-specific transaction helpers
- Users must remember to commit/rollback
- No automatic error handling

### Option B: MagicORM transaction helper (future)

```rust
db.transaction(|tx| async move {
    User::insert(&mut *tx, &user).await?;
    Post::insert(&mut *tx, &post).await?;
    Ok(())
}).await?;
```

**Pros:**
- Automatic commit on Ok, rollback on Err
- Cleaner error handling
- MagicORM-specific ergonomics

**Cons:**
- Another abstraction layer
- May not cover all use cases (nested transactions, savepoints)
- Adds complexity

## Decision

**For now: Option A (expose sqlx directly).**

Rationale:
1. The E2E tests demonstrate that sqlx transactions work correctly with MagicORM CRUD.
2. Adding a wrapper adds complexity without clear benefit for P1.
3. Thalos can build its own transaction patterns on top.
4. If Option B is needed later, it can be added without breaking changes.

## Future consideration

If Thalos needs transaction helpers, the API should be:

```rust
impl SqlitePool {
    pub async fn transaction<F, T, E>(&self, f: F) -> Result<T, E>
    where
        F: FnOnce(&mut sqlx::Transaction<'_, sqlx::Sqlite>) -> futures::Future<Output = Result<T, E>>,
    {
        let mut tx = self.begin().await?;
        let result = f(&mut tx).await?;
        tx.commit().await?;
        Ok(result)
    }
}
```

This can be added to `magic/src/sqlite.rs` when needed.

## Consequences

- No breaking changes
- Users must handle commit/rollback explicitly
- Transaction patterns are documented but not enforced
