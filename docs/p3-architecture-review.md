# P3: Architecture Review & Next Capabilities

## Status

Proposed — 2026-09-08

## Context

P0-P2 built the persistence engine. P3 should answer:

> **What kind of ORM is MagicORM now, and what capabilities are actually missing?**

## Current State

```
MagicORM
├── Connection/Pool (WAL, FK, busy_timeout)
├── CRUD (insert, get, put, delete)
├── Batch operations (insert_many)
├── Query builder (filters, order, limit, offset)
├── Query ergonomics (count, exists, null filters)
├── Index strategy (composite indexes)
├── Keyset pagination (cursor-based)
├── Streaming (lazy iteration)
├── Retention primitives (delete, vacuum, size)
└── Schema (create_all, introspect)
```

## Areas to Review

### 1. Schema Evolution

**Current state:**
- `ModelDescriptor` with columns, foreign_keys, indexes
- `create_all()` creates tables + indexes
- `describe_database()` introspects live schema
- `diff()` exists but not integrated

**What's missing:**
- Migration file generation from diff
- Migration apply/rollback tracking
- Schema versioning
- Column add/drop/change detection
- Index add/drop detection

**Priority:** High — needed for production use

### 2. Database Abstraction

**Current state:**
- SQLite fully working
- PostgreSQL compilation errors (pre-existing)
- `SqlDialect` trait exists
- Feature flags work

**What's missing:**
- PostgreSQL parity
- Dialect completeness
- Backend capability matrix

**Priority:** Medium — SQLite is sufficient for Thalos

### 3. ORM Ergonomics

**Current state:**
- Basic relations (HasMany, BelongsTo)
- Eager loading (with_many)
- Join support

**What's missing:**
- Typed query expressions (compile-time checked)
- Relation prefetching
- Aggregation helpers
- Transaction isolation levels

**Priority:** Low — current API is sufficient

### 4. Production Hardening

**Current state:**
- Error handling via anyhow
- Tracing instrumentation
- Basic tests

**What's missing:**
- Structured error types
- Metrics/observability
- Concurrency guarantees documentation
- Performance regression suite
- Fuzzing

**Priority:** Medium — needed before production use

## Recommended P3 Scope

### P3.1 — Schema Evolution (Core)

1. Migration diff engine integration
2. Migration file generation
3. Migration apply/rollback with tracking
4. Schema versioning

### P3.2 — Production Hardening

1. Structured error types
2. Performance regression test suite
3. Concurrency documentation

### P3.3 — PostgreSQL Parity (Optional)

1. Fix compilation errors
2. Dialect completeness
3. Cross-backend tests

## What NOT to do in P3

- Typed query expressions (complex, low ROI)
- Aggregation helpers (application-specific)
- Relation prefetching ( premature optimization)
- Transaction isolation levels (SQLx handles this)

## Architecture Decision

P3 should maintain the principle:

> **Workload justifies capability, not the other way around.**

Schema evolution is justified because:
- Thalos will evolve its data model
- Migrations are a universal need
- The infrastructure (ModelDescriptor, introspect, diff) already exists

PostgreSQL parity is optional because:
- Thalos uses SQLite
- Can be done later when needed
