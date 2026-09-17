# ADR 001: SQLite Connection First

## Status

Accepted — 2026-09-08

## Context

MagicORM is the persistence layer for Thalos, an edge-first robotics system. The edge computer (Raspberry Pi, Jetson, x86 gateway) runs a Rust runtime on Tokio that coordinates sensors, actuators, planning, and execution. SQLite is the durable operational store — not a cache, not a replica. The system must survive disconnection.

The current codebase has no connection or pool abstraction. Every callsite manually creates `SqlitePool`, runs PRAGMA statements, and manages lifecycle independently.

## Decision

**Connection is the primary abstraction. Pool is optional.**

```rust
// Primary API — single connection, single writer
let db = Sqlite::open("thalos.db").await?;
let db = Sqlite::open_memory().await?;

// Optional — only when concurrent readers are needed
let db = Sqlite::pool("thalos.db").await?;
```

### Why Connection First

1. **SQLite semantics favor single-writer.** A connection with WAL allows concurrent reads while writing. A pool adds complexity without proportional benefit for edge workloads.

2. **Edge devices have one writer.** The Thalos runtime writes observations, updates execution state, and manages configuration. There is no concurrent writer scenario.

3. **PRAGMA is per-connection.** `foreign_keys`, `busy_timeout`, `journal_mode` are connection-level settings. A pool must configure each connection on acquire — error-prone.

4. **Testing is simpler.** `open_memory()` returns a connection, no pool management.

### Configuration

```rust
pub struct SqliteConfig {
    pub journal_mode: JournalMode,  // WAL (default)
    pub foreign_keys: bool,         // true (default)
    pub busy_timeout: Duration,     // 5s (default)
    pub synchronous: Synchronous,   // Normal (default)
}
```

Presets:
- `SqliteConfig::default()` — WAL, FK on, busy=5s. For production edge.
- `SqliteConfig::in_memory()` — no WAL, FK on. For tests.

### Pool uses after_connect

```rust
Sqlite::pool("thalos.db").await?
// internally: after_connect callback applies SqliteConfig to each new connection
```

### What MagicORM does NOT do

- **No sync/replication** — that is Thalos Persistence layer, above the ORM.
- **No MCU support** — ESP32 sends `ExecutionSample` via transport, not SQLite.
- **No conflict resolution** — local SQLite is the source of truth.

## Consequences

- All existing callsites that manually create `SqlitePool` and run PRAGMAs must migrate to `Sqlite::open()`.
- Tests use `Sqlite::open_memory()` instead of manual pool + PRAGMA setup.
- Persistence test (open → write → close → reopen → verify) uses a real file, not in-memory.
- The `foreign_keys` PRAGMA is applied once per connection at initialization, not scattered across callsites.
