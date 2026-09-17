use magic_orm::{prelude::*, register_models};

#[derive(MagicModel, Debug)]
#[magic(table = "execution_sessions")]
pub struct ExecutionSession {
    pub id: i64,
    pub name: String,
    pub status: String,
    pub created_at: String,
}

#[derive(MagicModel, Debug)]
#[magic(table = "observations")]
pub struct Observation {
    pub id: i64,

    #[FK(ExecutionSession)]
    pub session_id: i64,

    pub sensor: String,
    pub value: f64,
    pub quality: String,
    pub timestamp: String,
}

has_many!(ExecutionSession => Observation);
register_models!(ExecutionSession, Observation);

#[tokio::test]
async fn thalos_lifecycle() {
    let db_path = "/tmp/thalos_e2e_test.db";

    // Limpiar DB previa
    let _ = std::fs::remove_file(db_path);

    // 1. Open SQLite con configuración edge
    let pool = Sqlite::pool(db_path).await.unwrap();

    // 2. Migrate (create tables)
    create_all::<_, AppModels>(&pool).await.unwrap();

    // 3. Create ExecutionSession
    let session_id = ExecutionSession::insert(
        &pool,
        &NewExecutionSession {
            name: "Test Session".to_string(),
            status: "running".to_string(),
            created_at: "2026-09-08T12:00:00Z".to_string(),
        },
    )
    .await
    .unwrap();
    assert!(session_id > 0);

    // 4. Insert Observations
    for i in 0..100 {
        Observation::insert(
            &pool,
            &NewObservation {
                session_id,
                sensor: format!("sensor_{}", i % 6),
                value: i as f64 * 0.1,
                quality: "good".to_string(),
                timestamp: format!("2026-09-08T12:00:{:02}Z", i),
            },
        )
        .await
        .unwrap();
    }

    // 5. Transaction: atomic batch
    {
        let mut tx = pool.begin().await.unwrap();
        for i in 100..200 {
            Observation::insert(
                &mut *tx,
                &NewObservation {
                    session_id,
                    sensor: format!("sensor_{}", i % 6),
                    value: i as f64 * 0.1,
                    quality: "good".to_string(),
                    timestamp: format!("2026-09-08T12:01:{:02}Z", i - 100),
                },
            )
            .await
            .unwrap();
        }
        tx.commit().await.unwrap();
    }

    // 6. Query observations
    let observations = Observation::query()
        .filter("session_id", "=", session_id)
        .order_by("timestamp", true)
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(observations.len(), 200);
    assert_eq!(observations[0].sensor, "sensor_0");
    assert_eq!(observations[199].sensor, "sensor_1");

    // 7. Query with filter_in
    let filtered = Observation::query()
        .filter_in("sensor", ["sensor_0", "sensor_1"])
        .fetch_all(&pool)
        .await
        .unwrap();
    assert!(filtered.len() > 0);
    assert!(
        filtered
            .iter()
            .all(|o| o.sensor == "sensor_0" || o.sensor == "sensor_1")
    );

    // 8. Count using raw SQL (count mode in QueryBuilder returns model type)
    let (count,): (i64,) = sqlx::query_as("SELECT count(*) FROM observations WHERE session_id = ?")
        .bind(session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 200);

    // 9. Get by id
    let fetched = Observation::get_by_id(&pool, observations[0].id)
        .await
        .unwrap();
    assert!(fetched.is_some());
    assert_eq!(fetched.unwrap().sensor, "sensor_0");

    // 10. HasMany relationship
    let sessions = ExecutionSession::query()
        .with_many::<Observation>()
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(sessions.parents.len(), 1);
    // children is a HashMap<SessionId, Vec<Observation>>, so len() is number of sessions with children
    let session_children = sessions.children.get(&session_id).unwrap();
    assert_eq!(session_children.len(), 200);

    // 11. Drop pool (close)
    drop(pool);

    // 12. Reopen + verify persistence
    {
        let pool = Sqlite::pool(db_path).await.unwrap();
        let (count,): (i64,) =
            sqlx::query_as("SELECT count(*) FROM observations WHERE session_id = ?")
                .bind(session_id)
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(count, 200);

        let session_fetched = ExecutionSession::get_by_id(&pool, session_id)
            .await
            .unwrap();
        assert!(session_fetched.is_some());
        assert_eq!(session_fetched.unwrap().name, "Test Session");
    }

    // Cleanup
    let _ = std::fs::remove_file(db_path);
    let _ = std::fs::remove_file(format!("{}-wal", db_path));
    let _ = std::fs::remove_file(format!("{}-shm", db_path));
}

#[tokio::test]
async fn thalos_in_memory_fast() {
    // Test rápido con in-memory para desarrollo
    let pool = Sqlite::pool_with_config(":memory:", SqliteConfig::in_memory())
        .await
        .unwrap();

    create_all::<_, AppModels>(&pool).await.unwrap();

    let session_id = ExecutionSession::insert(
        &pool,
        &NewExecutionSession {
            name: "Quick Test".to_string(),
            status: "done".to_string(),
            created_at: "2026-09-08T12:00:00Z".to_string(),
        },
    )
    .await
    .unwrap();

    for i in 0..10 {
        Observation::insert(
            &pool,
            &NewObservation {
                session_id,
                sensor: format!("s{}", i),
                value: i as f64,
                quality: "good".to_string(),
                timestamp: format!("2026-09-08T12:00:{:02}Z", i),
            },
        )
        .await
        .unwrap();
    }

    let (count,): (i64,) = sqlx::query_as("SELECT count(*) FROM observations WHERE session_id = ?")
        .bind(session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 10);
}

#[tokio::test]
async fn thalos_transaction_rollback() {
    let pool = Sqlite::pool_with_config(":memory:", SqliteConfig::in_memory())
        .await
        .unwrap();

    create_all::<_, AppModels>(&pool).await.unwrap();

    let session_id = ExecutionSession::insert(
        &pool,
        &NewExecutionSession {
            name: "Rollback Test".to_string(),
            status: "running".to_string(),
            created_at: "2026-09-08T12:00:00Z".to_string(),
        },
    )
    .await
    .unwrap();

    // Insert one observation
    Observation::insert(
        &pool,
        &NewObservation {
            session_id,
            sensor: "s0".to_string(),
            value: 0.0,
            quality: "good".to_string(),
            timestamp: "2026-09-08T12:00:00Z".to_string(),
        },
    )
    .await
    .unwrap();

    // Start transaction and rollback
    {
        let mut tx = pool.begin().await.unwrap();
        for i in 1..5 {
            Observation::insert(
                &mut *tx,
                &NewObservation {
                    session_id,
                    sensor: format!("s{}", i),
                    value: i as f64,
                    quality: "good".to_string(),
                    timestamp: format!("2026-09-08T12:00:{:02}Z", i),
                },
            )
            .await
            .unwrap();
        }
        tx.rollback().await.unwrap();
    }

    // Only the first observation should exist
    let (count,): (i64,) = sqlx::query_as("SELECT count(*) FROM observations WHERE session_id = ?")
        .bind(session_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count, 1);
}

#[tokio::test]
async fn thalos_update_and_delete() {
    let pool = Sqlite::pool_with_config(":memory:", SqliteConfig::in_memory())
        .await
        .unwrap();

    create_all::<_, AppModels>(&pool).await.unwrap();

    let session_id = ExecutionSession::insert(
        &pool,
        &NewExecutionSession {
            name: "Update Test".to_string(),
            status: "running".to_string(),
            created_at: "2026-09-08T12:00:00Z".to_string(),
        },
    )
    .await
    .unwrap();

    // Update session status
    ExecutionSession::put(
        &pool,
        session_id,
        &NewExecutionSession {
            name: "Update Test".to_string(),
            status: "completed".to_string(),
            created_at: "2026-09-08T12:00:00Z".to_string(),
        },
    )
    .await
    .unwrap();

    let fetched = ExecutionSession::get_by_id(&pool, session_id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(fetched.status, "completed");

    // Delete session
    let deleted = ExecutionSession::delete_by_id(&pool, session_id)
        .await
        .unwrap();
    assert_eq!(deleted, 1);

    let fetched = ExecutionSession::get_by_id(&pool, session_id)
        .await
        .unwrap();
    assert!(fetched.is_none());
}
