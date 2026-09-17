# Decision: Batch Insertion

## Status

Accepted — 2026-09-08

## Context

Thalos generates observation telemetry at scale:
- 6 channels × 1 Hz = 360 rows/min
- 20 channels × 10 Hz = 12,000 rows/min
- 100K rows/day at minimum

Individual INSERT statements are insufficient for this workload.

## Benchmark Results

| Scenario | 100 rows | 1K rows | 10K rows | 100K rows |
|----------|----------|---------|----------|-----------|
| Individual (no tx) | 9,950 rows/s | 9,009 rows/s | 9,768 rows/s | 9,627 rows/s |
| Individual in tx | 31,806 rows/s | 34,148 rows/s | 34,621 rows/s | 34,952 rows/s |
| Raw SQL prepared | 43,077 rows/s | 45,326 rows/s | 44,953 rows/s | 45,340 rows/s |
| Batch VALUES 100/stmt | 138,281 rows/s | 264,028 rows/s | 327,637 rows/s | 330,614 rows/s |

**Key findings:**
- Transaction wrapping: 3.5x speedup
- Prepared statements: 1.3x speedup
- Batch VALUES: 7-10x additional speedup
- Batch size 100 provides optimal performance for this workload

## Decision

**Implement `insert_many` with automatic chunking.**

### API

```rust
// For pool executors
Observation::insert_many(&pool, &observations).await?;

// For existing transactions
let mut tx = pool.begin().await?;
Observation::insert_many_in_tx(&mut tx, &observations).await?;
tx.commit().await?;
```

### Implementation

- Default batch size: 100 (configurable via `DEFAULT_BATCH_SIZE`)
- Generates batch INSERT SQL with multiple VALUES clauses
- Atomic: all rows inserted or none
- Works with both pool and transaction executors

### Usage in Thalos

```rust
// Observation ingestion
let observations: Vec<NewObservation> = ...;
Observation::insert_many(&pool, &observations).await?;

// Atomic batch with other operations
let mut tx = pool.begin().await?;
ExecutionSession::insert(&mut *tx, &session).await?;
Observation::insert_many_in_tx(&mut tx, &observations).await?;
tx.commit().await?;
```

## Consequences

- 4-5x throughput improvement for batch operations
- No breaking changes
- Atomicity preserved
- chunking is automatic and transparent
