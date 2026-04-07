use magic_orm::{prelude::*, register_models};
use sqlx::SqlitePool;

#[derive(MagicModel, Debug)]
#[magic(table = "users")]
pub struct User {
    pub id: i64,
    pub name: String,
    pub edad: i32,
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

#[derive(MagicModel, Debug)]
#[magic(table = "reactions")]
pub struct Reaction {
    pub id: i64,
    pub reaction_type: String,

    #[FK(Post)]
    pub post_id: i64,

    #[FK(User)]
    pub user_id: i64,
}

has_many!(User => Post, Reaction);
has_many!(Post => Reaction);

register_models!(User, Post, Reaction);

async fn setup_pool() -> SqlitePool {
    let mut pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::query("PRAGMA foreign_keys = ON;")
        .execute(&pool)
        .await
        .unwrap();

    create_all::<SqlitePool, AppModels>(&mut pool).await.unwrap();
    pool
}

#[tokio::test]
async fn test_transaction_success() {
    let pool = setup_pool().await;

    let mut tx = pool.begin().await.unwrap();

    let user_id = User::insert(&mut *tx, &NewUser {
        name: "Test".to_string(),
        edad: 20,
        email: "test@example.com".to_string(),
    }).await.unwrap();

    let post_id = Post::insert(&mut *tx, &NewPost {
        title: "Post".to_string(),
        content: "Content".to_string(),
        user_id,
    }).await.unwrap();

    let reaction_id = Reaction::insert(&mut *tx, &NewReaction {
        reaction_type: "like".to_string(),
        post_id,
        user_id,
    }).await.unwrap();

    tx.commit().await.unwrap();

    let fetched_user = User::get_by_id(&pool, user_id).await.unwrap().unwrap();
    assert_eq!(fetched_user.name, "Test");
}

#[tokio::test]
async fn test_transaction_failure() {
    let pool = setup_pool().await;

    let mut tx = pool.begin().await.unwrap();

    // Intentamos insertar un post con user_id inexistente
    let result = Post::insert(&mut *tx, &NewPost {
        title: "Fail Post".to_string(),
        content: "No user".to_string(),
        user_id: 999, // No existe
    }).await;

    assert!(result.is_err());

    // Commit no debería pasar, pero hacemos rollback explícito
    tx.rollback().await.unwrap();
}

#[tokio::test]
async fn test_delete_operations() {
    let pool = setup_pool().await;

    let user_id = User::insert(&pool, &NewUser {
        name: "ToDelete".to_string(),
        edad: 22,
        email: "del@example.com".to_string(),
    }).await.unwrap();

    let deleted = User::delete_by_id(&pool, user_id).await.unwrap();
    assert_eq!(deleted, 1);
}

#[tokio::test]
async fn test_has_many_relationship() {
    let pool = setup_pool().await;

    // Crear un usuario
    let user_id = User::insert(&pool, &NewUser {
        name: "RelTest".to_string(),
        edad: 30,
        email: "reltest@example.com".to_string(),
    }).await.unwrap();

    let user = User::get_by_id(&pool, user_id).await.unwrap().unwrap();

    // Crear un post asociado
    let post_id = Post::insert(&pool, &NewPost {
        title: "RelPost".to_string(),
        content: "Content".to_string(),
        user_id,
    }).await.unwrap();

    // Cargar los posts del usuario
    let fetched_posts = user.posts(&pool).await.unwrap();
    assert_eq!(fetched_posts.len(), 1);
    assert_eq!(fetched_posts[0].user_id, user_id);
}