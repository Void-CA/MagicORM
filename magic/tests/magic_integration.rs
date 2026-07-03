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

// Modelo con UUID como ID
#[derive(MagicModel, Debug)]
#[magic(table = "documents")]
pub struct Document {
    pub id: uuid::Uuid,
    pub title: String,
}

has_many!(User => Post, Reaction);
has_many!(Post => Reaction);

register_models!(User, Post, Reaction, Document);

async fn setup_pool() -> SqlitePool {
    let pool = SqlitePool::connect("sqlite::memory:").await.unwrap();
    sqlx::query("PRAGMA foreign_keys = ON;")
        .execute(&pool)
        .await
        .unwrap();

    create_all::<_, AppModels>(&pool).await.unwrap();
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

#[tokio::test]
async fn test_uuid_crud() {
    let pool = setup_pool().await;

    let doc_id = uuid::Uuid::new_v4();
    // Insert con UUID manual (el insert devuelve i64, no uuid::Uuid)
    sqlx::query("INSERT INTO documents (id, title) VALUES (?, ?)")
        .bind(doc_id)
        .bind("UUID Document")
        .execute(&pool)
        .await
        .unwrap();

    // Leer por UUID
    let doc = Document::get_by_id(&pool, doc_id).await.unwrap().unwrap();
    assert_eq!(doc.title, "UUID Document");

    // QueryBuilder con UUID
    let docs = Document::query()
        .filter("id", "=", doc_id)
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(docs.len(), 1);
    assert_eq!(docs[0].title, "UUID Document");
}

// =========================================================================
// Transacciones
// =========================================================================

#[tokio::test]
async fn test_transaction_insert_and_commit() {
    let pool = setup_pool().await;
    let mut tx = pool.begin().await.unwrap();

    let uid = User::insert(&mut *tx, &NewUser {
        name: "TxUser".to_string(), edad: 25, email: "tx@x.com".to_string(),
    }).await.unwrap();

    let pid = Post::insert(&mut *tx, &NewPost {
        title: "TxPost".to_string(), content: "C".to_string(), user_id: uid,
    }).await.unwrap();

    tx.commit().await.unwrap();

    // Verificar fuera de la transacción
    let user = User::get_by_id(&pool, uid).await.unwrap().unwrap();
    assert_eq!(user.name, "TxUser");

    let post = Post::get_by_id(&pool, pid).await.unwrap().unwrap();
    assert_eq!(post.user_id, uid);
}

#[tokio::test]
async fn test_transaction_rollback() {
    let pool = setup_pool().await;
    let mut tx = pool.begin().await.unwrap();

    let uid = User::insert(&mut *tx, &NewUser {
        name: "RollbackUser".to_string(), edad: 99, email: "rb@x.com".to_string(),
    }).await.unwrap();

    tx.rollback().await.unwrap();

    // No debería existir fuera de la transacción
    let user = User::get_by_id(&pool, uid).await.unwrap();
    assert!(user.is_none());
}

#[tokio::test]
async fn test_transaction_has_many_relation() {
    let pool = setup_pool().await;
    let mut tx = pool.begin().await.unwrap();

    let uid = User::insert(&mut *tx, &NewUser {
        name: "RelTx".to_string(), edad: 30, email: "reltx@x.com".to_string(),
    }).await.unwrap();

    Post::insert(&mut *tx, &NewPost {
        title: "P1".to_string(), content: "C1".to_string(), user_id: uid,
    }).await.unwrap();
    Post::insert(&mut *tx, &NewPost {
        title: "P2".to_string(), content: "C2".to_string(), user_id: uid,
    }).await.unwrap();

    // Cargar relación dentro de la misma transacción
    let user = User::get_by_id(&mut *tx, uid).await.unwrap().unwrap();
    let posts = user.posts(&mut *tx).await.unwrap();
    assert_eq!(posts.len(), 2);

    tx.commit().await.unwrap();
}

#[tokio::test]
async fn test_transaction_query_builder() {
    let pool = setup_pool().await;
    let mut tx = pool.begin().await.unwrap();

    let uid = User::insert(&mut *tx, &NewUser {
        name: "QBTx".to_string(), edad: 40, email: "qbtx@x.com".to_string(),
    }).await.unwrap();

    // QueryBuilder dentro de la transacción
    let users = User::query()
        .filter("name", "=", "QBTx")
        .fetch_all(&mut *tx)
        .await
        .unwrap();
    assert_eq!(users.len(), 1);
    assert_eq!(users[0].id, uid);

    // filter_in
    let users = User::query()
        .filter_in("id", [uid])
        .fetch_all(&mut *tx)
        .await
        .unwrap();
    assert_eq!(users.len(), 1);

    tx.commit().await.unwrap();
}

#[tokio::test]
async fn test_transaction_update_and_delete() {
    let pool = setup_pool().await;
    let mut tx = pool.begin().await.unwrap();

    let uid = User::insert(&mut *tx, &NewUser {
        name: "UpdDel".to_string(), edad: 50, email: "ud@x.com".to_string(),
    }).await.unwrap();

    // Update dentro de la transacción
    User::put(&mut *tx, uid, &NewUser {
        name: "Updated".to_string(), edad: 51, email: "ud2@x.com".to_string(),
    }).await.unwrap();

    let user = User::get_by_id(&mut *tx, uid).await.unwrap().unwrap();
    assert_eq!(user.name, "Updated");

    // Delete dentro de la transacción
    User::delete_by_id(&mut *tx, uid).await.unwrap();

    let user = User::get_by_id(&mut *tx, uid).await.unwrap();
    assert!(user.is_none());

    tx.rollback().await.unwrap();

    // Después del rollback, el user original debería existir
    let user = User::get_by_id(&pool, uid).await.unwrap().unwrap();
    assert_eq!(user.name, "UpdDel");
}