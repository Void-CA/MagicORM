# MagicORM

[![crates.io](https://img.shields.io/crates/v/magic_orm.svg)](https://crates.io/crates/magic_orm)
[![docs.rs](https://docs.rs/magic_orm/badge.svg)](https://docs.rs/magic_orm)
[![license](https://img.shields.io/badge/license-MIT-blue.svg)](https://github.com/Void-CA/MagicORM/blob/main/LICENSE)

A Rust ORM for SQL databases with a focus on ease of use, low boilerplate, and
performance.

MagicORM is built for **fast-evolving systems**: when models, relationships,
and fields change constantly, infrastructure should not dominate the code. It
uses strong conventions and automatic code generation so the domain model stays
the center of attention.

- **Backends:** SQLite (default) and PostgreSQL
- **Schema:** automatic table creation, introspection, and migrations
- **Queries:** typed `QueryBuilder`, filters, joins, eager loading
- **Scale:** batch insertion, streaming, keyset pagination, retention helpers

## Installation

```toml
[dependencies]
magic_orm = "0.2"
sqlx = { version = "0.8", features = ["runtime-tokio"] }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

SQLite is enabled by default. For PostgreSQL:

```toml
magic_orm = { version = "0.2", default-features = false, features = ["postgres"] }
```

## Quickstart

```rust,no_run
use magic_orm::{prelude::*, register_models};

#[derive(MagicModel, Debug)]
#[magic(table = "users")]
pub struct User {
    pub id: i64,
    pub name: String,
    pub edad: i32,
    pub email: String,
}

#[derive(MagicModel, Debug)]
#[magic(table = "posts")]
pub struct Post {
    pub id: i64,
    pub title: String,
    pub content: String,

    #[FK(User)]
    pub user_id: i64,
}

has_many!(User => Post);
register_models!(User, Post);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = Sqlite::pool("app.db").await?;

    // Create tables from the registered models.
    create_all::<_, AppModels>(&pool).await?;

    // Insert.
    let user_id = User::insert(
        &pool,
        &User::new("Alicia".into(), 25, "alicia@example.com".into()),
    )
    .await?;

    Post::insert(
        &pool,
        &Post::new("First post".into(), "Hello".into(), user_id),
    )
    .await?;

    // Fetch by id.
    let alicia = User::get_by_id(&pool, user_id).await?.unwrap();

    // Load relations.
    let posts = alicia.posts(&pool).await?;
    assert_eq!(posts.len(), 1);

    // Query builder.
    let found = User::query()
        .filter("edad", ">=", 18)
        .order_by("name", true)
        .fetch_all(&pool)
        .await?;
    println!("{} adult users", found.len());

    Ok(())
}
```

### Transactions

Transactions are exposed directly from `sqlx`:

```rust,no_run
# use magic_orm::{prelude::*, register_models};
# #[derive(MagicModel)] #[magic(table = "users")]
# pub struct User { pub id: i64, pub name: String }
# register_models!(User);
# async fn example(pool: SqlitePool) -> anyhow::Result<()> {
let mut tx = pool.begin().await?;
User::insert(&mut *tx, &User::new("Bob".into())).await?;
tx.commit().await?;
# Ok(()) }
```

### Batch insertion

```rust,ignore
User::insert_many(&pool, &users).await?;          // pool
User::insert_many_in_tx(&mut tx, &users).await?;  // existing transaction
```

### Upsert

```rust,ignore
User::upsert(&pool, &new_user).await?;          // conflict on PK, update rest
User::upsert_with_id(&pool, &user).await?;      // include id in the INSERT
User::upsert_many(&pool, &users).await?;        // batch
```

### Migrations

```bash
cargo install magic_cli   # (coming soon — currently built from source)
magic migrate new add_users
magic migrate generate add_users
magic migrate up
magic migrate status
magic migrate down
```

## Design Philosophy

MagicORM makes a deliberate trade-off: less structural freedom in exchange for
iteration speed, fewer repeated decisions, and less code affected by change.

| Reduced                        | Gained           |
|--------------------------------|------------------|
| Total structural flexibility   | Iteration speed  |
| Exhaustive configuration       | Simplicity       |
| Manual granular control        | Abstraction      |
| Explicit boilerplate           | Fluidity         |

See [docs/todo/database_abstraction_roadmap.md](https://github.com/Void-CA/MagicORM/blob/main/docs/todo/database_abstraction_roadmap.md)
for the capability roadmap and [docs/adr](https://github.com/Void-CA/MagicORM/tree/main/docs/adr) for architecture decisions.

## Status

MagicORM is under active development. The public API may change between minor
versions until `1.0`.

- P0 — SQLite persistence ✅
- P1 — ORM/runtime quality ✅
- P2 — Data lifecycle (indexes, cursors, streaming, retention) ✅
- P3 — Schema evolution (migrations, diff engine, rollback) ✅
- P3.5 — Production hardening (structured errors, perf regression, concurrency docs) 🚧

## License

Licensed under the [MIT License](https://github.com/Void-CA/MagicORM/blob/main/LICENSE).

Copyright (c) 2026 Ari Castillo &lt;castilloari282@gmail.com&gt;
