#![cfg(feature = "postgres")]

//! Integración con PostgreSQL.
//! Ejecutar: DATABASE_URL="postgres://localhost/magic_orm_test" cargo test --package magic_orm --test postgres_integration --no-default-features --features postgres

use magic_orm::{prelude::*, register_models};

#[derive(MagicModel, Debug)]
#[magic(table = "users")]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
}

#[derive(MagicModel, Debug)]
#[magic(table = "posts")]
pub struct Post {
    pub id: i64,
    pub title: String,
    pub content: String,
    #[FK(User)]
    pub user_id: i64,
}

register_models!(User, Post);

fn has_postgres() -> bool {
    std::env::var("DATABASE_URL")
        .map(|url| url.starts_with("postgres"))
        .unwrap_or(false)
}

async fn setup_pool() -> sqlx::PgPool {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL required");
    let pool = sqlx::PgPool::connect(&url).await.unwrap();

    sqlx::query("DROP TABLE IF EXISTS posts CASCADE")
        .execute(&pool)
        .await
        .unwrap();
    sqlx::query("DROP TABLE IF EXISTS users CASCADE")
        .execute(&pool)
        .await
        .unwrap();

    create_all::<_, AppModels>(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn test_postgres_crud() {
    if !has_postgres() {
        eprintln!("⚠ Skipping: set DATABASE_URL (postgres://...)");
        return;
    }
    let pool = setup_pool().await;

    let uid = User::insert(&pool, &NewUser {
        name: "Alice".to_string(), email: "alice@x.com".to_string(),
    }).await.unwrap();
    assert!(uid > 0);

    let user = User::get_by_id(&pool, uid).await.unwrap().unwrap();
    assert_eq!(user.name, "Alice");

    let pid = Post::insert(&pool, &NewPost {
        title: "P".to_string(), content: "C".to_string(), user_id: uid,
    }).await.unwrap();
    assert!(pid > 0);

    let posts = Post::query()
        .filter("user_id", "=", uid)
        .fetch_all(&pool).await.unwrap();
    assert_eq!(posts.len(), 1);

    assert_eq!(User::delete_by_id(&pool, uid).await.unwrap(), 1);
}

#[tokio::test]
async fn test_postgres_filter_in() {
    if !has_postgres() {
        eprintln!("⚠ Skipping: set DATABASE_URL");
        return;
    }
    let pool = setup_pool().await;
    let a = User::insert(&pool, &NewUser {
        name: "A".to_string(), email: "a@x.com".to_string(),
    }).await.unwrap();
    let b = User::insert(&pool, &NewUser {
        name: "B".to_string(), email: "b@x.com".to_string(),
    }).await.unwrap();

    assert_eq!(
        User::query().filter_in("id", [a, b])
            .fetch_all(&pool).await.unwrap().len(),
        2
    );
}

#[tokio::test]
async fn test_postgres_descriptors() {
    let descs = all_descriptors();
    assert_eq!(descs.len(), 2);
    assert!(descs.iter().any(|d| d.table == "users"));
    assert!(descs.iter().any(|d| d.table == "posts"));
}
