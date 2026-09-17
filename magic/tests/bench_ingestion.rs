use magic_orm::query::BindArg;
use magic_orm::{prelude::*, register_models};
use std::time::Instant;

#[derive(MagicModel, Debug)]
#[magic(table = "observations")]
pub struct Observation {
    pub id: i64,
    pub session_id: i64,
    pub sensor: String,
    pub value: f64,
    pub quality: String,
    pub timestamp: String,
}

register_models!(Observation);

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn new_obs(session_id: i64, i: usize, channel: usize) -> NewObservation {
    NewObservation {
        session_id,
        sensor: format!("channel_{}", channel),
        value: (i as f64) * 0.01,
        quality: "good".to_string(),
        timestamp: format!("2026-09-08T12:{:02}:{:02}.{:03}Z", i / 60, i % 60, i % 1000),
    }
}

struct BenchResult {
    name: String,
    count: usize,
    elapsed: std::time::Duration,
    rows_per_sec: f64,
}

impl BenchResult {
    fn report(&self) {
        println!(
            "  {:<45} {:>8} rows  {:>8.2}ms  {:>10.0} rows/s",
            self.name,
            self.count,
            self.elapsed.as_secs_f64() * 1000.0,
            self.rows_per_sec,
        );
    }
}

// ---------------------------------------------------------------------------
// Scenario A: Individual INSERT (no transaction)
// ---------------------------------------------------------------------------

async fn bench_insert_individual(pool: &sqlx::SqlitePool, count: usize) -> BenchResult {
    let start = Instant::now();
    for i in 0..count {
        Observation::insert(pool, &new_obs(1, i, i % 6))
            .await
            .unwrap();
    }
    let elapsed = start.elapsed();
    BenchResult {
        name: "A: INSERT individual (no tx)".to_string(),
        count,
        elapsed,
        rows_per_sec: count as f64 / elapsed.as_secs_f64(),
    }
}

// ---------------------------------------------------------------------------
// Scenario B: Individual INSERT inside transaction
// ---------------------------------------------------------------------------

async fn bench_insert_in_transaction(pool: &sqlx::SqlitePool, count: usize) -> BenchResult {
    let start = Instant::now();
    {
        let mut tx = pool.begin().await.unwrap();
        for i in 0..count {
            Observation::insert(&mut *tx, &new_obs(2, i, i % 6))
                .await
                .unwrap();
        }
        tx.commit().await.unwrap();
    }
    let elapsed = start.elapsed();
    BenchResult {
        name: "B: INSERT in transaction".to_string(),
        count,
        elapsed,
        rows_per_sec: count as f64 / elapsed.as_secs_f64(),
    }
}

// ---------------------------------------------------------------------------
// Scenario C: Raw SQL with prepared statement (sqlx::query)
// ---------------------------------------------------------------------------

async fn bench_insert_raw_prepared(pool: &sqlx::SqlitePool, count: usize) -> BenchResult {
    let start = Instant::now();
    {
        let mut tx = pool.begin().await.unwrap();
        for i in 0..count {
            sqlx::query("INSERT INTO observations (session_id, sensor, value, quality, timestamp) VALUES (?, ?, ?, ?, ?)")
                .bind(3i64)
                .bind(format!("channel_{}", i % 6))
                .bind((i as f64) * 0.01)
                .bind("good")
                .bind(format!("2026-09-08T12:{:02}:{:02}.{:03}Z", i / 60, i % 60, i % 1000))
                .execute(&mut *tx)
                .await
                .unwrap();
        }
        tx.commit().await.unwrap();
    }
    let elapsed = start.elapsed();
    BenchResult {
        name: "C: Raw SQL prepared (in tx)".to_string(),
        count,
        elapsed,
        rows_per_sec: count as f64 / elapsed.as_secs_f64(),
    }
}

// ---------------------------------------------------------------------------
// Scenario D: Batch via multiple VALUES (100 per statement)
// ---------------------------------------------------------------------------

async fn bench_insert_batch_values(pool: &sqlx::SqlitePool, count: usize) -> BenchResult {
    let batch_size = 100;
    let start = Instant::now();
    {
        let mut tx = pool.begin().await.unwrap();
        let mut offset = 0;
        while offset < count {
            let this_batch = (count - offset).min(batch_size);
            let mut sql =
                "INSERT INTO observations (session_id, sensor, value, quality, timestamp) VALUES "
                    .to_string();
            let mut bindings: Vec<(i64, String, f64, String, String)> = Vec::new();
            for i in 0..this_batch {
                if i > 0 {
                    sql.push(',');
                }
                sql.push_str("(?, ?, ?, ?, ?)");
                let idx = offset + i;
                bindings.push((
                    4i64,
                    format!("channel_{}", idx % 6),
                    (idx as f64) * 0.01,
                    "good".to_string(),
                    format!(
                        "2026-09-08T12:{:02}:{:02}.{:03}Z",
                        idx / 60,
                        idx % 60,
                        idx % 1000
                    ),
                ));
            }
            let mut q = sqlx::query(&sql);
            for (session_id, sensor, value, quality, timestamp) in &bindings {
                q = q
                    .bind(session_id)
                    .bind(sensor)
                    .bind(value)
                    .bind(quality)
                    .bind(timestamp);
            }
            q.execute(&mut *tx).await.unwrap();
            offset += this_batch;
        }
        tx.commit().await.unwrap();
    }
    let elapsed = start.elapsed();
    BenchResult {
        name: "D: Batch VALUES 100/stmt (in tx)".to_string(),
        count,
        elapsed,
        rows_per_sec: count as f64 / elapsed.as_secs_f64(),
    }
}

// ---------------------------------------------------------------------------
// Thalos workload scenarios
// ---------------------------------------------------------------------------

async fn bench_thalos_workload(
    pool: &sqlx::SqlitePool,
    channels: usize,
    hz: usize,
    duration_secs: usize,
) -> BenchResult {
    let total = channels * hz * duration_secs;
    let start = Instant::now();
    {
        let mut tx = pool.begin().await.unwrap();
        for sec in 0..duration_secs {
            for ch in 0..channels {
                for _ in 0..hz {
                    let i = sec * hz + ch;
                    Observation::insert(&mut *tx, &new_obs(100, i, ch))
                        .await
                        .unwrap();
                }
            }
        }
        tx.commit().await.unwrap();
    }
    let elapsed = start.elapsed();
    BenchResult {
        name: format!("Thalos: {}ch × {}Hz × {}s", channels, hz, duration_secs),
        count: total,
        elapsed,
        rows_per_sec: total as f64 / elapsed.as_secs_f64(),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[tokio::test]
async fn benchmark_observation_ingestion() {
    let pool = Sqlite::pool_with_config("/tmp/bench_ingestion.db", SqliteConfig::default())
        .await
        .unwrap();

    create_all::<_, AppModels>(&pool).await.unwrap();

    println!("\n{:=<60}", "");
    println!("  Observation Inestion Benchmark");
    println!("{:=<60}", "");

    println!("Scale benchmarks:");
    println!(
        "  {:<45} {:>8}      {:>8}      {:>10}",
        "Scenario", "Count", "Time", "Rows/s"
    );
    println!(
        "  {:<45} {:>8}      {:>8}      {:>10}",
        "-".repeat(45),
        "-".repeat(8),
        "-".repeat(8),
        "-".repeat(10)
    );

    for count in [100, 1_000, 10_000, 100_000] {
        // Clean table between runs
        sqlx::query("DELETE FROM observations")
            .execute(&pool)
            .await
            .unwrap();

        let r = bench_insert_individual(&pool, count).await;
        r.report();

        sqlx::query("DELETE FROM observations")
            .execute(&pool)
            .await
            .unwrap();

        let r = bench_insert_in_transaction(&pool, count).await;
        r.report();

        sqlx::query("DELETE FROM observations")
            .execute(&pool)
            .await
            .unwrap();

        let r = bench_insert_raw_prepared(&pool, count).await;
        r.report();

        sqlx::query("DELETE FROM observations")
            .execute(&pool)
            .await
            .unwrap();

        let r = bench_insert_batch_values(&pool, count).await;
        r.report();

        println!();
    }

    // Thalos workloads
    println!("Thalos workload benchmarks:");
    println!(
        "  {:<45} {:>8}      {:>8}      {:>10}",
        "Scenario", "Count", "Time", "Rows/s"
    );
    println!(
        "  {:<45} {:>8}      {:>8}      {:>10}",
        "-".repeat(45),
        "-".repeat(8),
        "-".repeat(8),
        "-".repeat(10)
    );

    for (ch, hz, dur) in [(6, 1, 60), (6, 10, 60), (20, 10, 60)] {
        sqlx::query("DELETE FROM observations")
            .execute(&pool)
            .await
            .unwrap();

        let r = bench_thalos_workload(&pool, ch, hz, dur).await;
        r.report();
    }

    // WAL impact
    println!("\nWAL impact:");
    let page_size: (i64,) = sqlx::query_as("PRAGMA page_size")
        .fetch_one(&pool)
        .await
        .unwrap();
    let page_count: (i64,) = sqlx::query_as("PRAGMA page_count")
        .fetch_one(&pool)
        .await
        .unwrap();
    let wal_checkpoint: (i64,) = sqlx::query_as("PRAGMA wal_checkpoint(PASSIVE)")
        .fetch_one(&pool)
        .await
        .unwrap();
    println!("  Page size: {} bytes", page_size.0);
    println!(
        "  Page count: {} ({:.2} MB)",
        page_count.0,
        page_count.0 as f64 * page_size.0 as f64 / 1_048_576.0
    );
    println!("  WAL checkpoint (freed pages): {}", wal_checkpoint.0);

    // Cleanup
    let _ = std::fs::remove_file("/tmp/bench_ingestion.db");
    let _ = std::fs::remove_file("/tmp/bench_ingestion.db-wal");
    let _ = std::fs::remove_file("/tmp/bench_ingestion.db-shm");
}

#[tokio::test]
async fn benchmark_memory_profile() {
    let pool = Sqlite::pool_with_config("/tmp/bench_memory.db", SqliteConfig::default())
        .await
        .unwrap();

    create_all::<_, AppModels>(&pool).await.unwrap();

    println!("\n{:=<60}", "");
    println!("  Memory Profile (single connection)");
    println!("{:=<60}", "");

    // Insert 10K observations
    let count = 10_000;
    {
        let mut tx = pool.begin().await.unwrap();
        for i in 0..count {
            Observation::insert(&mut *tx, &new_obs(1, i, i % 6))
                .await
                .unwrap();
        }
        tx.commit().await.unwrap();
    }

    // Check SQLite internal stats
    let freelist: (i64,) = sqlx::query_as("PRAGMA freelist_count")
        .fetch_one(&pool)
        .await
        .unwrap();
    let page_size: (i64,) = sqlx::query_as("PRAGMA page_size")
        .fetch_one(&pool)
        .await
        .unwrap();
    let page_count: (i64,) = sqlx::query_as("PRAGMA page_count")
        .fetch_one(&pool)
        .await
        .unwrap();

    println!("  After {} inserts:", count);
    println!("    Pages: {}", page_count.0);
    println!("    Page size: {} bytes", page_size.0);
    println!(
        "    File size: {:.2} MB",
        page_count.0 as f64 * page_size.0 as f64 / 1_048_576.0
    );
    println!("    Freelist: {} pages", freelist.0);

    // Cleanup
    let _ = std::fs::remove_file("/tmp/bench_memory.db");
    let _ = std::fs::remove_file("/tmp/bench_memory.db-wal");
    let _ = std::fs::remove_file("/tmp/bench_memory.db-shm");
}

// ---------------------------------------------------------------------------
// Regression benchmark: insert_many vs individual inserts
// ---------------------------------------------------------------------------

#[tokio::test]
async fn benchmark_insert_many_regression() {
    let pool = Sqlite::pool_with_config("/tmp/bench_insert_many.db", SqliteConfig::default())
        .await
        .unwrap();

    create_all::<_, AppModels>(&pool).await.unwrap();

    println!("\n{:=<60}", "");
    println!("  insert_many Regression Benchmark");
    println!("{:=<60}", "");

    println!(
        "\n  {:<45} {:>8}      {:>8}      {:>10}",
        "Scenario", "Count", "Time", "Rows/s"
    );
    println!(
        "  {:<45} {:>8}      {:>8}      {:>10}",
        "-".repeat(45),
        "-".repeat(8),
        "-".repeat(8),
        "-".repeat(10)
    );

    for count in [100, 1_000, 10_000] {
        // Individual inserts in transaction (baseline)
        sqlx::query("DELETE FROM observations")
            .execute(&pool)
            .await
            .unwrap();
        let start = Instant::now();
        {
            let mut tx = pool.begin().await.unwrap();
            for i in 0..count {
                Observation::insert(&mut *tx, &new_obs(1, i, i % 6))
                    .await
                    .unwrap();
            }
            tx.commit().await.unwrap();
        }
        let elapsed = start.elapsed();
        println!(
            "  {:<45} {:>8} rows  {:>8.2}ms  {:>10.0} rows/s",
            format!("Individual in tx (baseline)"),
            count,
            elapsed.as_secs_f64() * 1000.0,
            count as f64 / elapsed.as_secs_f64(),
        );

        // insert_many
        sqlx::query("DELETE FROM observations")
            .execute(&pool)
            .await
            .unwrap();
        let items: Vec<NewObservation> = (0..count).map(|i| new_obs(1, i, i % 6)).collect();
        let start = Instant::now();
        let inserted = Observation::insert_many(&pool, &items).await.unwrap();
        let elapsed = start.elapsed();
        assert_eq!(inserted, count as u64);
        println!(
            "  {:<45} {:>8} rows  {:>8.2}ms  {:>10.0} rows/s",
            format!("insert_many (batch)"),
            count,
            elapsed.as_secs_f64() * 1000.0,
            count as f64 / elapsed.as_secs_f64(),
        );

        println!();
    }

    // Cleanup
    let _ = std::fs::remove_file("/tmp/bench_insert_many.db");
    let _ = std::fs::remove_file("/tmp/bench_insert_many.db-wal");
    let _ = std::fs::remove_file("/tmp/bench_insert_many.db-shm");
}

// ---------------------------------------------------------------------------
// Temporal window query benchmark
// ---------------------------------------------------------------------------

#[tokio::test]
async fn benchmark_temporal_queries() {
    let pool = Sqlite::pool_with_config("/tmp/bench_temporal.db", SqliteConfig::default())
        .await
        .unwrap();

    create_all::<_, AppModels>(&pool).await.unwrap();

    // Insert 100K observations
    let count = 100_000;
    {
        let items: Vec<NewObservation> = (0..count).map(|i| new_obs(1, i, i % 6)).collect();
        Observation::insert_many(&pool, &items).await.unwrap();
    }

    println!("\n{:=<60}", "");
    println!("  Temporal Window Query Benchmark (100K observations)");
    println!("{:=<60}", "");

    println!(
        "\n  {:<45} {:>8}      {:>8}      {:>10}",
        "Query", "Rows", "Time", "Rows/s"
    );
    println!(
        "  {:<45} {:>8}      {:>8}      {:>10}",
        "-".repeat(45),
        "-".repeat(8),
        "-".repeat(8),
        "-".repeat(10)
    );

    // Q1: Full table scan
    let start = Instant::now();
    let rows = Observation::query().fetch_all(&pool).await.unwrap();
    let elapsed = start.elapsed();
    println!(
        "  {:<45} {:>8} rows  {:>8.2}ms  {:>10.0} rows/s",
        "Full table scan",
        rows.len(),
        elapsed.as_secs_f64() * 1000.0,
        rows.len() as f64 / elapsed.as_secs_f64(),
    );

    // Q2: Filter by session_id
    let start = Instant::now();
    let rows = Observation::query()
        .filter("session_id", "=", 1i64)
        .fetch_all(&pool)
        .await
        .unwrap();
    let elapsed = start.elapsed();
    println!(
        "  {:<45} {:>8} rows  {:>8.2}ms  {:>10.0} rows/s",
        "Filter by session_id",
        rows.len(),
        elapsed.as_secs_f64() * 1000.0,
        rows.len() as f64 / elapsed.as_secs_f64(),
    );

    // Q3: Time window (10% of data)
    let start = Instant::now();
    let rows = Observation::query()
        .filter("session_id", "=", 1i64)
        .filter("timestamp", ">=", "2026-09-08T12:16:40")
        .filter("timestamp", "<", "2026-09-08T12:33:20")
        .order_by("timestamp", true)
        .fetch_all(&pool)
        .await
        .unwrap();
    let elapsed = start.elapsed();
    println!(
        "  {:<45} {:>8} rows  {:>8.2}ms  {:>10.0} rows/s",
        "Time window (10%)",
        rows.len(),
        elapsed.as_secs_f64() * 1000.0,
        rows.len() as f64 / elapsed.as_secs_f64(),
    );

    // Q4: Time window with limit
    let start = Instant::now();
    let rows = Observation::query()
        .filter("session_id", "=", 1i64)
        .filter("timestamp", ">=", "2026-09-08T12:16:40")
        .filter("timestamp", "<", "2026-09-08T12:33:20")
        .order_by("timestamp", true)
        .limit(1000)
        .fetch_all(&pool)
        .await
        .unwrap();
    let elapsed = start.elapsed();
    println!(
        "  {:<45} {:>8} rows  {:>8.2}ms  {:>10.0} rows/s",
        "Time window + LIMIT 1000",
        rows.len(),
        elapsed.as_secs_f64() * 1000.0,
        rows.len() as f64 / elapsed.as_secs_f64(),
    );

    // Q5: Count
    let start = Instant::now();
    let count = Observation::query()
        .filter("session_id", "=", 1i64)
        .count()
        .fetch_count(&pool)
        .await
        .unwrap();
    let elapsed = start.elapsed();
    println!(
        "  {:<45} {:>8} rows  {:>8.2}ms  {:>10.0} rows/s",
        "Count by session_id",
        count,
        elapsed.as_secs_f64() * 1000.0,
        count as f64 / elapsed.as_secs_f64(),
    );

    // Q6: Exists
    let start = Instant::now();
    let exists = Observation::query()
        .filter("session_id", "=", 1i64)
        .exists()
        .fetch_exists(&pool)
        .await
        .unwrap();
    let elapsed = start.elapsed();
    println!(
        "  {:<45} {:>8}       {:>8.2}ms  {:>10}",
        "Exists by session_id",
        if exists { 1 } else { 0 },
        elapsed.as_secs_f64() * 1000.0,
        "-",
    );

    // Q7: Paginated (offset)
    let start = Instant::now();
    let rows = Observation::query()
        .filter("session_id", "=", 1i64)
        .order_by("timestamp", true)
        .limit(1000)
        .offset(50000)
        .fetch_all(&pool)
        .await
        .unwrap();
    let elapsed = start.elapsed();
    println!(
        "  {:<45} {:>8} rows  {:>8.2}ms  {:>10.0} rows/s",
        "Paginated (offset 50K, limit 1K)",
        rows.len(),
        elapsed.as_secs_f64() * 1000.0,
        rows.len() as f64 / elapsed.as_secs_f64(),
    );

    // Cleanup
    let _ = std::fs::remove_file("/tmp/bench_temporal.db");
    let _ = std::fs::remove_file("/tmp/bench_temporal.db-wal");
    let _ = std::fs::remove_file("/tmp/bench_temporal.db-shm");
}

// ---------------------------------------------------------------------------
// P2.1: Index strategy baseline benchmark
// ---------------------------------------------------------------------------

#[tokio::test]
async fn benchmark_index_baseline() {
    let pool = Sqlite::pool_with_config("/tmp/bench_index.db", SqliteConfig::default())
        .await
        .unwrap();

    create_all::<_, AppModels>(&pool).await.unwrap();

    // Insert 100K observations across 10 sessions
    let count = 100_000;
    let sessions = 10;
    {
        let items: Vec<NewObservation> = (0..count)
            .map(|i| new_obs((i % sessions) as i64 + 1, i, i % 6))
            .collect();
        Observation::insert_many(&pool, &items).await.unwrap();
    }

    println!("\n{:=<60}", "");
    println!("  P2.1 Index Strategy Baseline (100K obs, 10 sessions)");
    println!("{:=<60}", "");

    // Verify no indexes exist
    let indexes: Vec<(String,)> = sqlx::query_as(
        "SELECT name FROM sqlite_master WHERE type='index' AND tbl_name='observations'",
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    println!(
        "\n  Current indexes: {:?}",
        indexes.iter().map(|(n,)| n).collect::<Vec<_>>()
    );

    println!("\n  {:<50} {:>8}      {:>8}", "Query", "Rows", "Time");
    println!(
        "  {:<50} {:>8}      {:>8}",
        "-".repeat(50),
        "-".repeat(8),
        "-".repeat(8)
    );

    // Q1: WHERE session_id = ?
    let start = Instant::now();
    let rows = Observation::query()
        .filter("session_id", "=", 1i64)
        .fetch_all(&pool)
        .await
        .unwrap();
    let elapsed = start.elapsed();
    println!(
        "  {:<50} {:>8} rows  {:>8.2}ms",
        "Q1: WHERE session_id = ?",
        rows.len(),
        elapsed.as_secs_f64() * 1000.0,
    );

    // Q2: WHERE session_id = ? AND sampled_at BETWEEN ? AND ?
    let start = Instant::now();
    let rows = Observation::query()
        .filter("session_id", "=", 1i64)
        .filter("timestamp", ">=", "2026-09-08T12:16:40")
        .filter("timestamp", "<", "2026-09-08T12:33:20")
        .fetch_all(&pool)
        .await
        .unwrap();
    let elapsed = start.elapsed();
    println!(
        "  {:<50} {:>8} rows  {:>8.2}ms",
        "Q2: WHERE session_id AND time window",
        rows.len(),
        elapsed.as_secs_f64() * 1000.0,
    );

    // Q3: WHERE sampled_at BETWEEN ? AND ? (global)
    let start = Instant::now();
    let rows = Observation::query()
        .filter("timestamp", ">=", "2026-09-08T12:16:40")
        .filter("timestamp", "<", "2026-09-08T12:33:20")
        .fetch_all(&pool)
        .await
        .unwrap();
    let elapsed = start.elapsed();
    println!(
        "  {:<50} {:>8} rows  {:>8.2}ms",
        "Q3: WHERE time window (global)",
        rows.len(),
        elapsed.as_secs_f64() * 1000.0,
    );

    // Q4: ORDER BY sampled_at
    let start = Instant::now();
    let rows = Observation::query()
        .order_by("timestamp", true)
        .limit(1000)
        .fetch_all(&pool)
        .await
        .unwrap();
    let elapsed = start.elapsed();
    println!(
        "  {:<50} {:>8} rows  {:>8.2}ms",
        "Q4: ORDER BY timestamp LIMIT 1000",
        rows.len(),
        elapsed.as_secs_f64() * 1000.0,
    );

    // Q5: WHERE session_id = ? ORDER BY sampled_at
    let start = Instant::now();
    let rows = Observation::query()
        .filter("session_id", "=", 1i64)
        .order_by("timestamp", true)
        .limit(1000)
        .fetch_all(&pool)
        .await
        .unwrap();
    let elapsed = start.elapsed();
    println!(
        "  {:<50} {:>8} rows  {:>8.2}ms",
        "Q5: WHERE session_id ORDER BY timestamp",
        rows.len(),
        elapsed.as_secs_f64() * 1000.0,
    );

    // Q6: Latest observation for session
    let start = Instant::now();
    let rows = Observation::query()
        .filter("session_id", "=", 1i64)
        .order_by("timestamp", false)
        .limit(1)
        .fetch_all(&pool)
        .await
        .unwrap();
    let elapsed = start.elapsed();
    println!(
        "  {:<50} {:>8} rows  {:>8.2}ms",
        "Q6: Latest observation for session",
        rows.len(),
        elapsed.as_secs_f64() * 1000.0,
    );

    // Cleanup
    let _ = std::fs::remove_file("/tmp/bench_index.db");
    let _ = std::fs::remove_file("/tmp/bench_index.db-wal");
    let _ = std::fs::remove_file("/tmp/bench_index.db-shm");
}

// ---------------------------------------------------------------------------
// P2.1: Incremental index strategy benchmark
// ---------------------------------------------------------------------------

async fn run_queries(pool: &sqlx::SqlitePool) {
    println!("\n  {:<50} {:>8}      {:>8}", "Query", "Rows", "Time");
    println!(
        "  {:<50} {:>8}      {:>8}",
        "-".repeat(50),
        "-".repeat(8),
        "-".repeat(8)
    );

    // Q1: WHERE session_id = ?
    let start = Instant::now();
    let rows = Observation::query()
        .filter("session_id", "=", 1i64)
        .fetch_all(pool)
        .await
        .unwrap();
    let elapsed = start.elapsed();
    println!(
        "  {:<50} {:>8} rows  {:>8.2}ms",
        "Q1: WHERE session_id = ?",
        rows.len(),
        elapsed.as_secs_f64() * 1000.0,
    );

    // Q2: WHERE session_id = ? AND sampled_at BETWEEN ? AND ?
    let start = Instant::now();
    let rows = Observation::query()
        .filter("session_id", "=", 1i64)
        .filter("timestamp", ">=", "2026-09-08T12:16:40")
        .filter("timestamp", "<", "2026-09-08T12:33:20")
        .fetch_all(pool)
        .await
        .unwrap();
    let elapsed = start.elapsed();
    println!(
        "  {:<50} {:>8} rows  {:>8.2}ms",
        "Q2: WHERE session_id AND time window",
        rows.len(),
        elapsed.as_secs_f64() * 1000.0,
    );

    // Q3: WHERE sampled_at BETWEEN ? AND ? (global)
    let start = Instant::now();
    let rows = Observation::query()
        .filter("timestamp", ">=", "2026-09-08T12:16:40")
        .filter("timestamp", "<", "2026-09-08T12:33:20")
        .fetch_all(pool)
        .await
        .unwrap();
    let elapsed = start.elapsed();
    println!(
        "  {:<50} {:>8} rows  {:>8.2}ms",
        "Q3: WHERE time window (global)",
        rows.len(),
        elapsed.as_secs_f64() * 1000.0,
    );

    // Q4: ORDER BY sampled_at LIMIT 1000
    let start = Instant::now();
    let rows = Observation::query()
        .order_by("timestamp", true)
        .limit(1000)
        .fetch_all(pool)
        .await
        .unwrap();
    let elapsed = start.elapsed();
    println!(
        "  {:<50} {:>8} rows  {:>8.2}ms",
        "Q4: ORDER BY timestamp LIMIT 1000",
        rows.len(),
        elapsed.as_secs_f64() * 1000.0,
    );

    // Q5: WHERE session_id = ? ORDER BY sampled_at LIMIT 1000
    let start = Instant::now();
    let rows = Observation::query()
        .filter("session_id", "=", 1i64)
        .order_by("timestamp", true)
        .limit(1000)
        .fetch_all(pool)
        .await
        .unwrap();
    let elapsed = start.elapsed();
    println!(
        "  {:<50} {:>8} rows  {:>8.2}ms",
        "Q5: WHERE session_id ORDER BY timestamp",
        rows.len(),
        elapsed.as_secs_f64() * 1000.0,
    );

    // Q6: Latest observation for session
    let start = Instant::now();
    let rows = Observation::query()
        .filter("session_id", "=", 1i64)
        .order_by("timestamp", false)
        .limit(1)
        .fetch_all(pool)
        .await
        .unwrap();
    let elapsed = start.elapsed();
    println!(
        "  {:<50} {:>8} rows  {:>8.2}ms",
        "Q6: Latest observation for session",
        rows.len(),
        elapsed.as_secs_f64() * 1000.0,
    );
}

async fn measure_insert_throughput(pool: &sqlx::SqlitePool, count: usize) -> f64 {
    let items: Vec<NewObservation> = (0..count)
        .map(|i| new_obs((i % 10) as i64 + 1, i, i % 6))
        .collect();
    let start = Instant::now();
    Observation::insert_many(pool, &items).await.unwrap();
    let elapsed = start.elapsed();
    count as f64 / elapsed.as_secs_f64()
}

async fn explain_queries(pool: &sqlx::SqlitePool) {
    println!("\n  EXPLAIN QUERY PLAN:");
    println!("  {}", "-".repeat(60));

    let queries = [
        ("Q1", "SELECT * FROM observations WHERE session_id = 1"),
        (
            "Q2",
            "SELECT * FROM observations WHERE session_id = 1 AND timestamp >= '2026-09-08T12:16:40' AND timestamp < '2026-09-08T12:33:20'",
        ),
        (
            "Q3",
            "SELECT * FROM observations WHERE timestamp >= '2026-09-08T12:16:40' AND timestamp < '2026-09-08T23:33:20'",
        ),
        (
            "Q4",
            "SELECT * FROM observations ORDER BY timestamp ASC LIMIT 1000",
        ),
        (
            "Q5",
            "SELECT * FROM observations WHERE session_id = 1 ORDER BY timestamp ASC LIMIT 1000",
        ),
        (
            "Q6",
            "SELECT * FROM observations WHERE session_id = 1 ORDER BY timestamp DESC LIMIT 1",
        ),
    ];

    for (name, sql) in queries {
        let rows: Vec<(i32, i32, i32, String)> =
            sqlx::query_as(&format!("EXPLAIN QUERY PLAN {}", sql))
                .fetch_all(pool)
                .await
                .unwrap();
        let detail: Vec<&str> = rows.iter().map(|(_, _, _, d)| d.as_str()).collect();
        println!("  {}: {}", name, detail.join(" -> "));
    }
}

#[tokio::test]
async fn benchmark_p2_1_index_strategies() {
    println!("\n{:=<70}", "");
    println!("  P2.1 Index Strategy Benchmark (100K obs, 10 sessions)");
    println!("{:=<70}", "");

    // =====================================================================
    // Baseline (no indexes)
    // =====================================================================
    println!("\n{:=<70}", "");
    println!("  BASELINE (no indexes)");
    println!("{:=<70}", "");

    let pool = Sqlite::pool_with_config("/tmp/p21_baseline.db", SqliteConfig::default())
        .await
        .unwrap();
    create_all::<_, AppModels>(&pool).await.unwrap();

    // Insert data
    let insert_count = 100_000;
    {
        let items: Vec<NewObservation> = (0..insert_count)
            .map(|i| new_obs((i % 10) as i64 + 1, i, i % 6))
            .collect();
        Observation::insert_many(&pool, &items).await.unwrap();
    }

    let baseline_insert = measure_insert_throughput(&pool, 1000).await;
    println!("\n  Insert throughput: {:.0} rows/s", baseline_insert);
    run_queries(&pool).await;
    explain_queries(&pool).await;

    let _ = std::fs::remove_file("/tmp/p21_baseline.db");
    let _ = std::fs::remove_file("/tmp/p21_baseline.db-wal");
    let _ = std::fs::remove_file("/tmp/p21_baseline.db-shm");

    // =====================================================================
    // Strategy A: (session_id)
    // =====================================================================
    println!("\n{:=<70}", "");
    println!("  STRATEGY A: (session_id)");
    println!("{:=<70}", "");

    let pool = Sqlite::pool_with_config("/tmp/p21_strategy_a.db", SqliteConfig::default())
        .await
        .unwrap();
    create_all::<_, AppModels>(&pool).await.unwrap();

    // Insert data
    {
        let items: Vec<NewObservation> = (0..insert_count)
            .map(|i| new_obs((i % 10) as i64 + 1, i, i % 6))
            .collect();
        Observation::insert_many(&pool, &items).await.unwrap();
    }

    // Create index
    sqlx::query("CREATE INDEX idx_obs_session ON observations(session_id)")
        .execute(&pool)
        .await
        .unwrap();

    let strategy_a_insert = measure_insert_throughput(&pool, 1000).await;
    println!(
        "\n  Insert throughput: {:.0} rows/s (baseline: {:.0})",
        strategy_a_insert, baseline_insert
    );
    run_queries(&pool).await;
    explain_queries(&pool).await;

    let _ = std::fs::remove_file("/tmp/p21_strategy_a.db");
    let _ = std::fs::remove_file("/tmp/p21_strategy_a.db-wal");
    let _ = std::fs::remove_file("/tmp/p21_strategy_a.db-shm");

    // =====================================================================
    // Strategy B: (sampled_at)
    // =====================================================================
    println!("\n{:=<70}", "");
    println!("  STRATEGY B: (sampled_at)");
    println!("{:=<70}", "");

    let pool = Sqlite::pool_with_config("/tmp/p21_strategy_b.db", SqliteConfig::default())
        .await
        .unwrap();
    create_all::<_, AppModels>(&pool).await.unwrap();

    // Insert data
    {
        let items: Vec<NewObservation> = (0..insert_count)
            .map(|i| new_obs((i % 10) as i64 + 1, i, i % 6))
            .collect();
        Observation::insert_many(&pool, &items).await.unwrap();
    }

    // Create index
    sqlx::query("CREATE INDEX idx_obs_sampled_at ON observations(timestamp)")
        .execute(&pool)
        .await
        .unwrap();

    let strategy_b_insert = measure_insert_throughput(&pool, 1000).await;
    println!(
        "\n  Insert throughput: {:.0} rows/s (baseline: {:.0})",
        strategy_b_insert, baseline_insert
    );
    run_queries(&pool).await;
    explain_queries(&pool).await;

    let _ = std::fs::remove_file("/tmp/p21_strategy_b.db");
    let _ = std::fs::remove_file("/tmp/p21_strategy_b.db-wal");
    let _ = std::fs::remove_file("/tmp/p21_strategy_b.db-shm");

    // =====================================================================
    // Strategy C: (session_id, sampled_at)
    // =====================================================================
    println!("\n{:=<70}", "");
    println!("  STRATEGY C: (session_id, sampled_at)");
    println!("{:=<70}", "");

    let pool = Sqlite::pool_with_config("/tmp/p21_strategy_c.db", SqliteConfig::default())
        .await
        .unwrap();
    create_all::<_, AppModels>(&pool).await.unwrap();

    // Insert data
    {
        let items: Vec<NewObservation> = (0..insert_count)
            .map(|i| new_obs((i % 10) as i64 + 1, i, i % 6))
            .collect();
        Observation::insert_many(&pool, &items).await.unwrap();
    }

    // Create index
    sqlx::query("CREATE INDEX idx_obs_session_time ON observations(session_id, timestamp)")
        .execute(&pool)
        .await
        .unwrap();

    let strategy_c_insert = measure_insert_throughput(&pool, 1000).await;
    println!(
        "\n  Insert throughput: {:.0} rows/s (baseline: {:.0})",
        strategy_c_insert, baseline_insert
    );
    run_queries(&pool).await;
    explain_queries(&pool).await;

    let _ = std::fs::remove_file("/tmp/p21_strategy_c.db");
    let _ = std::fs::remove_file("/tmp/p21_strategy_c.db-wal");
    let _ = std::fs::remove_file("/tmp/p21_strategy_c.db-shm");

    // =====================================================================
    // Summary
    // =====================================================================
    println!("\n{:=<70}", "");
    println!("  SUMMARY");
    println!("{:=<70}", "");
    println!("\n  Insert throughput comparison:");
    println!("    Baseline:  {:.0} rows/s", baseline_insert);
    println!(
        "    A (session): {:.0} rows/s ({:.1}x)",
        strategy_a_insert,
        strategy_a_insert / baseline_insert
    );
    println!(
        "    B (time):    {:.0} rows/s ({:.1}x)",
        strategy_b_insert,
        strategy_b_insert / baseline_insert
    );
    println!(
        "    C (both):    {:.0} rows/s ({:.1}x)",
        strategy_c_insert,
        strategy_c_insert / baseline_insert
    );
}

// ---------------------------------------------------------------------------
// P2.2: Keyset vs OFFSET benchmark
// ---------------------------------------------------------------------------

#[tokio::test]
async fn benchmark_keyset_vs_offset() {
    let pool = Sqlite::pool_with_config("/tmp/bench_keyset.db", SqliteConfig::default())
        .await
        .unwrap();

    create_all::<_, AppModels>(&pool).await.unwrap();

    // Insert 100K observations with index
    let count = 100_000;
    {
        let items: Vec<NewObservation> = (0..count).map(|i| new_obs(1, i, i % 6)).collect();
        Observation::insert_many(&pool, &items).await.unwrap();
    }

    // Create the composite index
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_obs_session_time ON observations(session_id, timestamp)",
    )
    .execute(&pool)
    .await
    .unwrap();

    println!("\n{:=<70}", "");
    println!("  P2.2 Keyset vs OFFSET Benchmark (100K observations)");
    println!("{:=<70}", "");

    println!(
        "\n  {:<40} {:>10}      {:>10}",
        "Position", "OFFSET", "Keyset"
    );
    println!(
        "  {:<40} {:>10}      {:>10}",
        "-".repeat(40),
        "-".repeat(10),
        "-".repeat(10)
    );

    let page_size = 1000u32;

    // Test different positions
    for offset in [0, 10_000, 50_000, 90_000] {
        // OFFSET approach
        let start = Instant::now();
        let rows = Observation::query()
            .filter("session_id", "=", 1i64)
            .order_by("timestamp", true)
            .order_by("id", true)
            .limit(page_size)
            .offset(offset)
            .fetch_all(&pool)
            .await
            .unwrap();
        let offset_time = start.elapsed();

        // Get cursor values from the last row of previous page (or start from beginning)
        let cursor = if offset == 0 {
            // First page: start from minimum values
            magic_orm::query::Cursor::new(vec![
                BindArg::Text("2026-09-08T00:00:00.000Z".into()),
                BindArg::I64(0),
            ])
        } else {
            // Get the last row of previous page to create cursor
            let prev_rows = Observation::query()
                .filter("session_id", "=", 1i64)
                .order_by("timestamp", true)
                .order_by("id", true)
                .limit(1)
                .offset(offset - 1)
                .fetch_all(&pool)
                .await
                .unwrap();

            if let Some(last) = prev_rows.last() {
                magic_orm::query::Cursor::new(vec![
                    BindArg::Text(last.timestamp.clone()),
                    BindArg::I64(last.id),
                ])
            } else {
                magic_orm::query::Cursor::new(vec![
                    BindArg::Text("2026-09-08T00:00:00.000Z".into()),
                    BindArg::I64(0),
                ])
            }
        };

        // Keyset approach
        let start = Instant::now();
        let rows_keyset = Observation::query()
            .filter("session_id", "=", 1i64)
            .order_by("timestamp", true)
            .order_by("id", true)
            .limit(page_size)
            .after(cursor)
            .fetch_all(&pool)
            .await
            .unwrap();
        let keyset_time = start.elapsed();

        println!(
            "  {:<40} {:>8.2}ms      {:>8.2}ms",
            format!("offset = {}", offset),
            offset_time.as_secs_f64() * 1000.0,
            keyset_time.as_secs_f64() * 1000.0,
        );
    }

    // Cleanup
    let _ = std::fs::remove_file("/tmp/bench_keyset.db");
    let _ = std::fs::remove_file("/tmp/bench_keyset.db-wal");
    let _ = std::fs::remove_file("/tmp/bench_keyset.db-shm");
}

// ---------------------------------------------------------------------------
// P2.3: Streaming vs fetch_all benchmark
// ---------------------------------------------------------------------------

#[tokio::test]
async fn benchmark_streaming_vs_fetch_all() {
    use futures::StreamExt;

    let pool = Sqlite::pool_with_config("/tmp/bench_streaming.db", SqliteConfig::default())
        .await
        .unwrap();

    create_all::<_, AppModels>(&pool).await.unwrap();

    // Insert 100K observations
    let count = 100_000;
    {
        let items: Vec<NewObservation> = (0..count).map(|i| new_obs(1, i, i % 6)).collect();
        Observation::insert_many(&pool, &items).await.unwrap();
    }

    println!("\n{:=<70}", "");
    println!("  P2.3 Streaming vs fetch_all Benchmark (100K observations)");
    println!("{:=<70}", "");

    println!("\n  {:<40} {:>10}      {:>10}", "Method", "Time", "Rows/s");
    println!(
        "  {:<40} {:>10}      {:>10}",
        "-".repeat(40),
        "-".repeat(10),
        "-".repeat(10)
    );

    // fetch_all
    let start = Instant::now();
    let rows = Observation::query()
        .filter("session_id", "=", 1i64)
        .order_by("timestamp", true)
        .fetch_all(&pool)
        .await
        .unwrap();
    let elapsed = start.elapsed();
    println!(
        "  {:<40} {:>8.2}ms  {:>10.0} rows/s",
        "fetch_all (materialized)",
        elapsed.as_secs_f64() * 1000.0,
        rows.len() as f64 / elapsed.as_secs_f64(),
    );

    // stream (count only)
    let start = Instant::now();
    let mut stream = Observation::query()
        .filter("session_id", "=", 1i64)
        .order_by("timestamp", true)
        .stream(&pool)
        .unwrap();
    let mut stream_count = 0;
    while let Some(row) = stream.next().await {
        row.unwrap();
        stream_count += 1;
    }
    let elapsed = start.elapsed();
    println!(
        "  {:<40} {:>8.2}ms  {:>10.0} rows/s",
        "stream (lazy iteration)",
        elapsed.as_secs_f64() * 1000.0,
        stream_count as f64 / elapsed.as_secs_f64(),
    );

    // stream with take (first 1000 only)
    let start = Instant::now();
    let mut stream = Observation::query()
        .filter("session_id", "=", 1i64)
        .order_by("timestamp", true)
        .stream(&pool)
        .unwrap();
    let mut partial_count = 0;
    while let Some(_row) = stream.next().await {
        _row.unwrap();
        partial_count += 1;
        if partial_count >= 1000 {
            break;
        }
    }
    let elapsed = start.elapsed();
    println!(
        "  {:<40} {:>8.2}ms  {:>10} rows",
        "stream (first 1000 only)",
        elapsed.as_secs_f64() * 1000.0,
        partial_count,
    );

    // Cleanup
    let _ = std::fs::remove_file("/tmp/bench_streaming.db");
    let _ = std::fs::remove_file("/tmp/bench_streaming.db-wal");
    let _ = std::fs::remove_file("/tmp/bench_streaming.db-shm");
}
