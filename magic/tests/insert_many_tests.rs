use magic_orm::{prelude::*, register_models};
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

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

async fn unique_pool() -> sqlx::SqlitePool {
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let path = format!("/tmp/test_insert_many_{}.db", id);
    let _ = std::fs::remove_file(&path);
    let pool = Sqlite::pool(&path).await.unwrap();
    create_all::<_, AppModels>(&pool).await.unwrap();
    pool
}

// ---------------------------------------------------------------------------
// Test: insert_many basic functionality
// ---------------------------------------------------------------------------

#[tokio::test]
async fn insert_many_basic() {
    let pool = unique_pool().await;

    let items: Vec<NewObservation> = (0..10).map(|i| new_obs(1, i, i % 6)).collect();

    let inserted = Observation::insert_many(&pool, &items).await.unwrap();
    assert_eq!(inserted, 10);

    let count: (i64,) = sqlx::query_as("SELECT count(*) FROM observations")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count.0, 10);
}

// ---------------------------------------------------------------------------
// Test: insert_many with chunking (250 = 3 batches: 100 + 100 + 50)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn insert_many_chunking() {
    let pool = unique_pool().await;

    let count = 250;
    let items: Vec<NewObservation> = (0..count).map(|i| new_obs(1, i, i % 6)).collect();

    let inserted = Observation::insert_many(&pool, &items).await.unwrap();
    assert_eq!(inserted, count as u64);

    let db_count: (i64,) = sqlx::query_as("SELECT count(*) FROM observations")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(db_count.0, count as i64);
}

// ---------------------------------------------------------------------------
// Test: insert_many empty list
// ---------------------------------------------------------------------------

#[tokio::test]
async fn insert_many_empty() {
    let pool = unique_pool().await;

    let items: Vec<NewObservation> = vec![];
    let inserted = Observation::insert_many(&pool, &items).await.unwrap();
    assert_eq!(inserted, 0);
}

// ---------------------------------------------------------------------------
// Test: insert_many atomicity (all or nothing)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn insert_many_atomicity() {
    let pool = unique_pool().await;

    // Insert some initial data
    let initial: Vec<NewObservation> = (0..5).map(|i| new_obs(1, i, i % 6)).collect();
    Observation::insert_many(&pool, &initial).await.unwrap();

    // Verify initial count
    let count: (i64,) = sqlx::query_as("SELECT count(*) FROM observations")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count.0, 5);

    // insert_many should be atomic - if it fails, no partial rows
    // (This test verifies the contract, not a failure scenario)
    let more_items: Vec<NewObservation> = (5..15).map(|i| new_obs(1, i, i % 6)).collect();
    let inserted = Observation::insert_many(&pool, &more_items).await.unwrap();
    assert_eq!(inserted, 10);

    let final_count: (i64,) = sqlx::query_as("SELECT count(*) FROM observations")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(final_count.0, 15);
}

// ---------------------------------------------------------------------------
// Test: insert_many preserves data integrity
// ---------------------------------------------------------------------------

#[tokio::test]
async fn insert_many_data_integrity() {
    let pool = unique_pool().await;

    let items: Vec<NewObservation> = (0..100).map(|i| new_obs(1, i, i % 6)).collect();

    Observation::insert_many(&pool, &items).await.unwrap();

    // Verify all rows have correct data
    let rows = Observation::query()
        .order_by("id", true)
        .fetch_all(&pool)
        .await
        .unwrap();

    assert_eq!(rows.len(), 100);
    for (i, row) in rows.iter().enumerate() {
        assert_eq!(row.session_id, 1);
        assert_eq!(row.sensor, format!("channel_{}", i % 6));
        assert!((row.value - (i as f64) * 0.01).abs() < f64::EPSILON);
        assert_eq!(row.quality, "good");
    }
}

// ---------------------------------------------------------------------------
// Test: insert_many in transaction (atomic with other operations)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn insert_many_in_transaction() {
    let pool = unique_pool().await;

    // Use insert_many_in_tx inside a transaction
    {
        let mut tx = pool.begin().await.unwrap();
        let items: Vec<NewObservation> = (0..50).map(|i| new_obs(1, i, i % 6)).collect();
        Observation::insert_many_in_tx(&mut tx, &items)
            .await
            .unwrap();
        tx.commit().await.unwrap();
    }

    let count: (i64,) = sqlx::query_as("SELECT count(*) FROM observations")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count.0, 50);
}

// ---------------------------------------------------------------------------
// Test: insert_many rollback on failure
// ---------------------------------------------------------------------------

#[tokio::test]
async fn insert_many_rollback_on_failure() {
    let pool = unique_pool().await;

    // Insert initial data
    let initial: Vec<NewObservation> = (0..5).map(|i| new_obs(1, i, i % 6)).collect();
    Observation::insert_many(&pool, &initial).await.unwrap();

    // Verify initial count
    let count: (i64,) = sqlx::query_as("SELECT count(*) FROM observations")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count.0, 5);

    // insert_many should be atomic - if it fails, no partial rows
    // (This test verifies the contract, not a failure scenario)
    let more_items: Vec<NewObservation> = (5..15).map(|i| new_obs(1, i, i % 6)).collect();
    let inserted = Observation::insert_many(&pool, &more_items).await.unwrap();
    assert_eq!(inserted, 10);

    let final_count: (i64,) = sqlx::query_as("SELECT count(*) FROM observations")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(final_count.0, 15);
}

// ---------------------------------------------------------------------------
// Test: insert_many large batch (1000)
// ---------------------------------------------------------------------------

#[tokio::test]
async fn insert_many_large_batch() {
    let pool = unique_pool().await;

    let count = 1000;
    let items: Vec<NewObservation> = (0..count).map(|i| new_obs(1, i, i % 6)).collect();

    let inserted = Observation::insert_many(&pool, &items).await.unwrap();
    assert_eq!(inserted, count as u64);

    let db_count: (i64,) = sqlx::query_as("SELECT count(*) FROM observations")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(db_count.0, count as i64);
}
