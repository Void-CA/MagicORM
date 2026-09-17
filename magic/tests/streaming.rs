use futures::StreamExt;
use magic_orm::{prelude::*, register_models};
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(MagicModel, Debug, Clone)]
#[magic(table = "streaming_observations")]
pub struct StreamingObservation {
    pub id: i64,
    pub session_id: i64,
    pub value: f64,
}

register_models!(StreamingObservation);

fn new_obs(session_id: i64, i: usize) -> NewStreamingObservation {
    NewStreamingObservation {
        session_id,
        value: (i as f64) * 0.1,
    }
}

async fn unique_pool() -> sqlx::SqlitePool {
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let path = format!("/tmp/test_streaming_{}.db", id);
    let _ = std::fs::remove_file(&path);
    let pool = Sqlite::pool(&path).await.unwrap();
    create_all::<_, AppModels>(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn stream_basic() {
    let pool = unique_pool().await;

    let items: Vec<_> = (0..100).map(|i| new_obs(1, i)).collect();
    StreamingObservation::insert_many(&pool, &items)
        .await
        .unwrap();

    let mut stream = StreamingObservation::query()
        .filter("session_id", "=", 1i64)
        .order_by("id", true)
        .stream(&pool)
        .unwrap();

    let mut count = 0;
    while let Some(row) = stream.next().await {
        let obs = row.unwrap();
        assert_eq!(obs.session_id, 1);
        count += 1;
    }
    assert_eq!(count, 100);
}

#[tokio::test]
async fn stream_with_limit() {
    let pool = unique_pool().await;

    let items: Vec<_> = (0..100).map(|i| new_obs(1, i)).collect();
    StreamingObservation::insert_many(&pool, &items)
        .await
        .unwrap();

    let mut stream = StreamingObservation::query()
        .filter("session_id", "=", 1i64)
        .order_by("id", true)
        .limit(10)
        .stream(&pool)
        .unwrap();

    let mut count = 0;
    while let Some(row) = stream.next().await {
        row.unwrap();
        count += 1;
    }
    assert_eq!(count, 10);
}

#[tokio::test]
async fn stream_matches_fetch_all() {
    let pool = unique_pool().await;

    let items: Vec<_> = (0..50).map(|i| new_obs(1, i)).collect();
    StreamingObservation::insert_many(&pool, &items)
        .await
        .unwrap();

    // fetch_all
    let all = StreamingObservation::query()
        .filter("session_id", "=", 1i64)
        .order_by("id", true)
        .fetch_all(&pool)
        .await
        .unwrap();

    // stream
    let mut stream = StreamingObservation::query()
        .filter("session_id", "=", 1i64)
        .order_by("id", true)
        .stream(&pool)
        .unwrap();

    let mut streamed = Vec::new();
    while let Some(row) = stream.next().await {
        streamed.push(row.unwrap());
    }

    assert_eq!(all.len(), streamed.len());
    for (a, b) in all.iter().zip(streamed.iter()) {
        assert_eq!(a.id, b.id);
        assert_eq!(a.value, b.value);
    }
}

#[tokio::test]
async fn stream_empty_result() {
    let pool = unique_pool().await;

    let mut stream = StreamingObservation::query()
        .filter("session_id", "=", 999i64)
        .stream(&pool)
        .unwrap();

    let mut count = 0;
    while let Some(row) = stream.next().await {
        row.unwrap();
        count += 1;
    }
    assert_eq!(count, 0);
}

#[tokio::test]
async fn stream_in_transaction() {
    let pool = unique_pool().await;

    let items: Vec<_> = (0..20).map(|i| new_obs(1, i)).collect();
    StreamingObservation::insert_many(&pool, &items)
        .await
        .unwrap();

    let mut tx = pool.begin().await.unwrap();

    let count = {
        let mut stream = StreamingObservation::query()
            .filter("session_id", "=", 1i64)
            .order_by("id", true)
            .stream(&mut *tx)
            .unwrap();

        let mut count = 0;
        while let Some(row) = stream.next().await {
            row.unwrap();
            count += 1;
        }
        count
    };

    assert_eq!(count, 20);
    tx.commit().await.unwrap();
}
