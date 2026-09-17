use magic_orm::{prelude::*, register_models};

#[derive(MagicModel, Debug)]
#[magic(table = "test_observations")]
#[magic(index(
    name = "idx_test_obs_session_time",
    columns = ["session_id", "timestamp"]
))]
pub struct TestObservation {
    pub id: i64,
    pub session_id: i64,
    pub timestamp: String,
    pub value: f64,
}

register_models!(TestObservation);

#[tokio::test]
async fn create_all_creates_indexes() {
    let pool = Sqlite::pool_with_config(":memory:", SqliteConfig::in_memory())
        .await
        .unwrap();

    // First create_all should create table and index
    create_all::<_, AppModels>(&pool).await.unwrap();

    // Verify index exists in SQLite catalog
    let indexes: Vec<(String, String)> = sqlx::query_as(
        "SELECT name, sql FROM sqlite_master WHERE type='index' AND tbl_name='test_observations'",
    )
    .fetch_all(&pool)
    .await
    .unwrap();

    assert!(!indexes.is_empty(), "Expected at least one index");

    let (name, sql) = &indexes[0];
    assert_eq!(name, "idx_test_obs_session_time");
    assert!(sql.contains("session_id"));
    assert!(sql.contains("timestamp"));

    println!("Index created: {} -> {}", name, sql);
}

#[tokio::test]
async fn create_all_is_idempotent() {
    let pool = Sqlite::pool_with_config(":memory:", SqliteConfig::in_memory())
        .await
        .unwrap();

    // First create_all
    create_all::<_, AppModels>(&pool).await.unwrap();

    // Second create_all should not fail or duplicate indexes
    create_all::<_, AppModels>(&pool).await.unwrap();

    // Verify only one index exists (not duplicated)
    let count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM sqlite_master WHERE type='index' AND tbl_name='test_observations'",
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(count.0, 1, "Expected exactly 1 index, found {}", count.0);
}

#[tokio::test]
async fn index_metadata_matches_sql() {
    // Verify that the derive macro generates correct metadata
    let indexes = <TestObservation as ModelMeta>::indexes();
    assert_eq!(indexes.len(), 1);
    assert_eq!(indexes[0].name, "idx_test_obs_session_time");
    assert_eq!(indexes[0].columns, vec!["session_id", "timestamp"]);
    assert!(!indexes[0].unique);

    // Verify DDL generation matches
    let sql = magic_orm::schema::create::create_index_sql("test_observations", &indexes[0]);
    assert!(sql.contains("CREATE INDEX IF NOT EXISTS idx_test_obs_session_time"));
    assert!(sql.contains("(session_id, timestamp)"));
}

#[tokio::test]
async fn index_in_descriptor() {
    let desc = <TestObservation as Describe>::descriptor();
    assert_eq!(desc.indexes.len(), 1);
    assert_eq!(desc.indexes[0].name, "idx_test_obs_session_time");
    assert_eq!(desc.indexes[0].columns, vec!["session_id", "timestamp"]);
}
