use magic_orm::query::{BindArg, Cursor};
use magic_orm::{prelude::*, register_models};
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

#[derive(MagicModel, Debug, Clone)]
#[magic(table = "pagination_observations")]
#[magic(index(
    name = "idx_pagination_obs",
    columns = ["session_id", "timestamp"]
))]
pub struct PaginationObservation {
    pub id: i64,
    pub session_id: i64,
    pub timestamp: String,
    pub value: f64,
}

register_models!(PaginationObservation);

fn new_obs(session_id: i64, i: usize) -> NewPaginationObservation {
    NewPaginationObservation {
        session_id,
        timestamp: format!("2026-09-08T12:{:02}:{:02}.{:03}Z", i / 60, i % 60, i % 1000),
        value: (i as f64) * 0.1,
    }
}

async fn unique_pool() -> sqlx::SqlitePool {
    let id = COUNTER.fetch_add(1, Ordering::SeqCst);
    let path = format!("/tmp/test_keyset_{}.db", id);
    let _ = std::fs::remove_file(&path);
    let pool = Sqlite::pool(&path).await.unwrap();
    create_all::<_, AppModels>(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn keyset_pagination_full_scan() {
    let pool = unique_pool().await;

    // Insert 1000 observations
    let items: Vec<_> = (0..1000).map(|i| new_obs(1, i)).collect();
    PaginationObservation::insert_many(&pool, &items)
        .await
        .unwrap();

    // Fetch all at once
    let all = PaginationObservation::query()
        .filter("session_id", "=", 1i64)
        .order_by("timestamp", true)
        .order_by("id", true)
        .fetch_all(&pool)
        .await
        .unwrap();

    assert_eq!(all.len(), 1000);

    // Paginate with keyset
    let page_size = 100;
    let mut collected = Vec::new();
    let mut cursor: Option<Cursor> = None;

    loop {
        let mut query = PaginationObservation::query()
            .filter("session_id", "=", 1i64)
            .order_by("timestamp", true)
            .order_by("id", true)
            .limit(page_size);

        if let Some(c) = cursor.take() {
            query = query.after(c);
        }

        let page = query.fetch_all(&pool).await.unwrap();
        let page_len = page.len();

        if let Some(last) = page.last() {
            cursor = Some(Cursor::new(vec![
                BindArg::Text(last.timestamp.clone()),
                BindArg::I64(last.id),
            ]));
        }

        collected.extend(page);

        if page_len < page_size as usize {
            break;
        }
    }

    assert_eq!(collected.len(), 1000);

    // Verify order matches
    for i in 0..1000 {
        assert_eq!(collected[i].id, all[i].id);
        assert_eq!(collected[i].timestamp, all[i].timestamp);
    }
}

#[tokio::test]
async fn keyset_pagination_empty_result() {
    let pool = unique_pool().await;

    // Cursor past all data
    let rows = PaginationObservation::query()
        .filter("session_id", "=", 1i64)
        .order_by("timestamp", true)
        .order_by("id", true)
        .limit(100)
        .after(Cursor::new(vec![
            BindArg::Text("9999-01-01T00:00:00.000Z".into()),
            BindArg::I64(i64::MAX),
        ]))
        .fetch_all(&pool)
        .await
        .unwrap();

    assert!(rows.is_empty());
}

#[tokio::test]
async fn keyset_pagination_with_duplicates() {
    let pool = unique_pool().await;

    // Insert observations with same timestamp
    let same_time = "2026-09-08T12:00:00.000Z";
    for i in 0..10 {
        PaginationObservation::insert(
            &pool,
            &NewPaginationObservation {
                session_id: 1,
                timestamp: same_time.to_string(),
                value: i as f64,
            },
        )
        .await
        .unwrap();
    }

    // All should have same timestamp but different IDs
    let all = PaginationObservation::query()
        .filter("session_id", "=", 1i64)
        .order_by("timestamp", true)
        .order_by("id", true)
        .fetch_all(&pool)
        .await
        .unwrap();

    assert_eq!(all.len(), 10);
    // All timestamps are same
    assert!(all.iter().all(|o| o.timestamp == same_time));
    // IDs are unique
    let ids: Vec<i64> = all.iter().map(|o| o.id).collect();
    let mut sorted_ids = ids.clone();
    sorted_ids.sort();
    sorted_ids.dedup();
    assert_eq!(sorted_ids.len(), 10);

    // Paginate - cursor should handle duplicates correctly
    let page1 = PaginationObservation::query()
        .filter("session_id", "=", 1i64)
        .order_by("timestamp", true)
        .order_by("id", true)
        .limit(5)
        .fetch_all(&pool)
        .await
        .unwrap();

    assert_eq!(page1.len(), 5);

    let last = page1.last().unwrap();
    let page2 = PaginationObservation::query()
        .filter("session_id", "=", 1i64)
        .order_by("timestamp", true)
        .order_by("id", true)
        .limit(5)
        .after(Cursor::new(vec![
            BindArg::Text(last.timestamp.clone()),
            BindArg::I64(last.id),
        ]))
        .fetch_all(&pool)
        .await
        .unwrap();

    assert_eq!(page2.len(), 5);

    // No overlap between pages
    let page1_ids: Vec<i64> = page1.iter().map(|o| o.id).collect();
    let page2_ids: Vec<i64> = page2.iter().map(|o| o.id).collect();
    assert!(page1_ids.iter().all(|id| !page2_ids.contains(id)));
}

#[tokio::test]
async fn keyset_pagination_desc_order() {
    let pool = unique_pool().await;

    let items: Vec<_> = (0..100).map(|i| new_obs(1, i)).collect();
    PaginationObservation::insert_many(&pool, &items)
        .await
        .unwrap();

    // DESC order
    let all = PaginationObservation::query()
        .filter("session_id", "=", 1i64)
        .order_by("timestamp", false)
        .order_by("id", false)
        .fetch_all(&pool)
        .await
        .unwrap();

    // Paginate DESC
    let page1 = PaginationObservation::query()
        .filter("session_id", "=", 1i64)
        .order_by("timestamp", false)
        .order_by("id", false)
        .limit(30)
        .fetch_all(&pool)
        .await
        .unwrap();

    let last = page1.last().unwrap();
    let page2 = PaginationObservation::query()
        .filter("session_id", "=", 1i64)
        .order_by("timestamp", false)
        .order_by("id", false)
        .limit(30)
        .after(Cursor::new(vec![
            BindArg::Text(last.timestamp.clone()),
            BindArg::I64(last.id),
        ]))
        .fetch_all(&pool)
        .await
        .unwrap();

    // Verify all pages combined = all rows
    let mut combined = page1.clone();
    combined.extend(page2);
    assert_eq!(combined.len(), 60);

    // Verify no duplicates
    let ids: Vec<i64> = combined.iter().map(|o| o.id).collect();
    let mut sorted_ids = ids.clone();
    sorted_ids.sort();
    sorted_ids.dedup();
    assert_eq!(sorted_ids.len(), 60);
}

#[tokio::test]
async fn keyset_pagination_empty_dataset() {
    let pool = unique_pool().await;

    let rows = PaginationObservation::query()
        .filter("session_id", "=", 1i64)
        .order_by("timestamp", true)
        .order_by("id", true)
        .limit(100)
        .fetch_all(&pool)
        .await
        .unwrap();

    assert!(rows.is_empty());
}
