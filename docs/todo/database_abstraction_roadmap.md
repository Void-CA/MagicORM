# MagicORM Roadmap

## Current State (Sep 2026)

### P0 — Local SQLite persistence ✅ COMPLETED

- Connection API (`Sqlite::open`)
- Pool API (`Sqlite::pool`)
- WAL + SQLite configuration (`SqliteConfig`)
- Foreign keys (automated via config)
- busy_timeout (5s default)
- In-memory support (`SqliteConfig::in_memory()`)
- Dependency cleanup (`sqlx` with `default-features = false`)
- Thalos lifecycle E2E (open → migrate → CRUD → transaction → query → close → reopen → verify)
- Persistence across reopen verified
- Transactions (sqlx native, works with CRUD)

**Known issues:**
- 2 pre-existing `magic_integration` failures (`test_transaction_rollback`, `test_transaction_update_and_delete`)
- `cargo check --no-default-features --features postgres` fails (duplicate function names in `impl_crud!`, `last_insert_rowid()` used for Postgres)
- Not introduced by SQLite connection work
- Must remain tracked independently

---

### P1 — ORM/runtime quality ✅ COMPLETED

#### P1.1 — Runtime agnosticism ✅
Audit direct `tokio::*` usage in `magic/`, `magic_cli/`, `tests/`, `examples/`.
Goal: no runtime assumptions beyond what sqlx requires.

**Result:** `tokio` moved to dev-dependencies. Library has no direct runtime coupling.

#### P1.2 — Minimal feature graph ✅
Verify `--no-default-features --features postgres` compiles cleanly.
Review Cargo.lock / feature graph / binary size.

**Result:** SQLite compiles cleanly. Postgres has pre-existing compilation errors (duplicate function names in `impl_crud!`).

#### P1.3 — Bulk ingestion ✅
Benchmark observation ingestion workload for Thalos.

**Results:**
- Transaction wrapping: 3.5x speedup (9K → 35K rows/s)
- Prepared statements: 1.3x speedup (35K → 45K rows/s)
- Batch VALUES 100/stmt: 7-10x speedup (45K → 330K rows/s)
- Thalos workload (20ch × 10Hz): 348ms for 12K rows

**Implementation:** `insert_many` added to CRUD module with automatic chunking (batch size 100).
- `insert_many(&pool, items)` — for pool executors
- `insert_many_in_tx(&mut tx, items)` — for existing transactions
- Atomic: all rows inserted or none
- Tests: 8/8 passing, regression benchmark shows 4-5x improvement

#### P1.4 — Transaction API decision ✅
Document whether MagicORM exposes sqlx transactions directly or adds a wrapper.

**Decision:** Expose sqlx transactions directly for now (ADR-002).

#### P1.5 — Query ergonomics ✅
Review current QueryBuilder API surface before adding features.
Ensure basic read patterns are coherent.

**Implemented:**
- `fetch_count(executor) -> Result<i64>` — for count queries
- `fetch_exists(executor) -> Result<bool>` — for existence checks
- `order_by(col, asc)` — accumulates multiple order clauses
- `filter_null(col)` / `filter_not_null(col)` — NULL checks
- `exists()` mode — generates `SELECT 1 ... LIMIT 1`

**Benchmark results (100K observations):**
- Full scan: 594ms (168K rows/s)
- Filter by session_id: 569ms (176K rows/s)
- Time window (10%): 99ms (113K rows/s)
- Time window + LIMIT: 42ms (24K rows/s)
- COUNT: 17ms (6M rows/s)
- EXISTS: 0.45ms
- OFFSET pagination: 189ms (5K rows/s) — **known limitation, moved to P2**

**Conclusion:** Query model is sufficient for Thalos immediate workload.

---

### P2 — Data lifecycle ✅ COMPLETE

| Phase | Status | Key capability |
|-------|--------|----------------|
| P2.1 Index strategy | ✅ | `#[magic(index(...))]`, composite indexes |
| P2.2 Keyset/cursor pagination | ✅ | `Cursor`, `after()`, constant-time pagination |
| P2.3 Large-result streaming | ✅ | `stream()`, lazy iteration |
| P2.4 Retention/archival | ✅ | `delete()`, `vacuum()`, `database_size()` |

**Result:** MagicORM now handles Thalos-scale telemetry workloads:
- 188K rows/s insert, 330K rows/s batch
- 0.37ms latest observation query
- ~6ms keyset pagination (constant)
- 73ms stream first 1K (9x faster than materializing)

---

### P3 — Schema evolution (NEXT)

Architecture review before implementation.

#### P3.1 — Capability audit
Review what MagicORM is now and what's missing:
- Schema evolution (migrations, diff engine)
- Database abstraction (PostgreSQL parity)
- ORM ergonomics (relations, joins, eager/lazy)
- Production hardening (error model, observability)

#### P3.2 — Schema model
Define schema representation:
```text
ModelDescriptor
├── columns
├── foreign_keys
└── indexes
```
→ Schema representation → Database introspection → Diff → Migration

#### P3.3 — Migration diff engine
Detect schema changes and generate migrations.

#### P3.4 — Migration apply/rollback
Apply and rollback migrations with versioning.

---

### P3 — Distributed / specialized

- libSQL/Turso adapter
- Replication
- Conflict resolution
- Embedded/MCU experimentation (out of scope for Thalos)

---

## Architecture Decisions

### ADR 001: SQLite Connection First
`docs/adr/001-sqlite-connection-first.md`

Key decisions:
- Connection is the primary abstraction, Pool is optional
- `Sqlite::open() → SqliteConnection`
- `Sqlite::pool() → SqlitePool`
- PRAGMAs applied per-connection at initialization
- No sync/replication in ORM core
- No MCU support (ESP32 sends via transport, not SQLite)

### ADR 002: Transaction API
`docs/adr/002-transaction-api.md`

Key decisions:
- Expose sqlx transactions directly for now
- No wrapper abstraction needed
- `insert_many_in_tx` available for batch operations within transactions

### ADR 003: Batch Insertion
`docs/adr/003-batch-insertion.md`

Key decisions:
- `insert_many` added for telemetry-scale workloads
- Default batch size: 100 (based on benchmark results)
- Atomic: all rows inserted or none
- Two variants: `insert_many(&pool, items)` and `insert_many_in_tx(&mut tx, items)`
- Batch insertion is required for observation ingestion at scale

---

## Pre-existing Issues (tracked separately)

| Test | Failure | Introduced |
|------|---------|------------|
| `test_transaction_rollback` | User exists after rollback | Pre-P0 |
| `test_transaction_update_and_delete` | User name mismatch after rollback | Pre-P0 |

These are not blocking P1 work but must be investigated and resolved independently.
