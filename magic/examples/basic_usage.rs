/// Ejemplo básico de uso de magic_orm.
/// Ejecutar con: `cargo run --example basic_usage`
use magic_orm::{prelude::*, register_models};

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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut pool = SqlitePool::connect("sqlite://test.db").await?;
    sqlx::query("PRAGMA foreign_keys = ON;")
        .execute(&pool)
        .await?;

    create_all::<SqlitePool, AppModels>(&mut pool).await?;

    let new_user = User::new("Alicia".into(), 25, "alicia@example.com".into());

    let insert_id = User::insert(& pool, &new_user).await?;

    let post_1 = Post::new("Primer post".into(), "Contenido del primer post".into(), insert_id);
    let post_2 = Post::new("Segundo post".into(), "Contenido del segundo post".into(), insert_id);
    Post::insert(& pool, &post_1).await?;
    Post::insert(& pool, &post_2).await?;


    let alicia = User::get_by_id(&pool, insert_id).await?.unwrap();
    let mut tx = pool.begin().await?;
    let posts = alicia.posts(&mut *tx).await?;
    tx.commit().await?;
    
    let x = User::query().with_many::<Post>().fetch_all(&pool).await?;
    println!("{:#?}", x);
    Ok(())
}
