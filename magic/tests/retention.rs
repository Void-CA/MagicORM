use magic_orm::{prelude::*, register_models};
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(MagicModel, Debug, Clone)]
#[magic(table = "retention_observations")]
pub struct RetentionObservation {
    pub id: i64,
    pub session_id: i64,
    pub timestamp: String,
    pub value: f64,
}

register_models!(RetentionObservation);

fn new_obs(session_id: i64, i: usize) -> NewRetentionObservation {
    NewRetentionObservation {
        session_id,
        timestamp: format!("2026-09-08T12:{:02}:{:02}.{:03}Z", i / 60, i % 60, i % 1000),
        value: (i as f64) * 0.1,
    }
}

async fn unique_pool() -> sqlx::SqlitePool {
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let path = format!("/tmp/test_retention_{}.db", id);
    let _ = std::fs::remove_file(&path);
    let pool = Sqlite::pool(&path).await.unwrap();
    create_all::<_, AppModels>(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn delete_basic() {
    let pool = unique_pool().await;

    let items: Vec<_> = (0..100).map(|i| new_obs(1, i)).collect();
    RetentionObservation::insert_many(&pool, &items)
        .await
        .unwrap();

    let count_before: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM retention_observations")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count_before.0, 100);

    // Delete observations with id > 50
    let deleted = RetentionObservation::query()
        .filter("id", ">", 50i64)
        .delete(&pool)
        .await
        .unwrap();

    assert_eq!(deleted, 50);

    let count_after: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM retention_observations")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count_after.0, 50);
}

#[tokio::test]
async fn delete_with_temporal_filter() {
    let pool = unique_pool().await;

    let items: Vec<_> = (0..100).map(|i| new_obs(1, i)).collect();
    RetentionObservation::insert_many(&pool, &items)
        .await
        .unwrap();

    // Delete old observations (before 12:30)
    let deleted = RetentionObservation::query()
        .filter("timestamp", "<", "2026-09-08T12:30:00.000Z")
        .delete(&pool)
        .await
        .unwrap();

    assert!(deleted > 0);

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM retention_observations")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert!(count.0 < 100);
}

#[tokio::test]
async fn delete_no_match() {
    let pool = unique_pool().await;

    let items: Vec<_> = (0..10).map(|i| new_obs(1, i)).collect();
    RetentionObservation::insert_many(&pool, &items)
        .await
        .unwrap();

    let deleted = RetentionObservation::query()
        .filter("session_id", "=", 999i64)
        .delete(&pool)
        .await
        .unwrap();

    assert_eq!(deleted, 0);

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM retention_observations")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count.0, 10);
}

#[tokio::test]
async fn delete_in_transaction() {
    let pool = unique_pool().await;

    let items: Vec<_> = (0..100).map(|i| new_obs(1, i)).collect();
    RetentionObservation::insert_many(&pool, &items)
        .await
        .unwrap();

    // Delete in transaction, then rollback
    {
        let mut tx = pool.begin().await.unwrap();
        RetentionObservation::query()
            .filter("id", ">", 50i64)
            .delete(&mut *tx)
            .await
            .unwrap();
        tx.rollback().await.unwrap();
    }

    // Nothing should be deleted
    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM retention_observations")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count.0, 100);
}

#[tokio::test]
async fn delete_in_transaction_commit() {
    let pool = unique_pool().await;

    let items: Vec<_> = (0..100).map(|i| new_obs(1, i)).collect();
    RetentionObservation::insert_many(&pool, &items)
        .await
        .unwrap();

    // Delete in transaction, then commit
    {
        let mut tx = pool.begin().await.unwrap();
        RetentionObservation::query()
            .filter("id", ">", 50i64)
            .delete(&mut *tx)
            .await
            .unwrap();
        tx.commit().await.unwrap();
    }

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM retention_observations")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count.0, 50);
}

#[tokio::test]
async fn sqlite_vacuum() {
    let pool = unique_pool().await;

    let items: Vec<_> = (0..1000).map(|i| new_obs(1, i)).collect();
    RetentionObservation::insert_many(&pool, &items)
        .await
        .unwrap();

    // Delete most data
    RetentionObservation::query()
        .filter("id", ">", 100i64)
        .delete(&pool)
        .await
        .unwrap();

    let size_before = Sqlite::database_size(&pool).await.unwrap();
    let freelist_before = Sqlite::freelist_count(&pool).await.unwrap();
    assert!(freelist_before > 0, "Should have free pages after deletion");

    // VACUUM
    Sqlite::vacuum(&pool).await.unwrap();

    let size_after = Sqlite::database_size(&pool).await.unwrap();
    let freelist_after = Sqlite::freelist_count(&pool).await.unwrap();

    assert_eq!(freelist_after, 0, "VACUUM should eliminate free pages");
    assert!(
        size_after < size_before,
        "Database should be smaller after VACUUM"
    );
}

#[tokio::test]
async fn sqlite_database_size() {
    let pool = unique_pool().await;

    let size = Sqlite::database_size(&pool).await.unwrap();
    assert!(size > 0, "Database should have non-zero size");

    let freelist = Sqlite::freelist_count(&pool).await.unwrap();
    assert_eq!(freelist, 0, "Fresh database should have no free pages");
}
