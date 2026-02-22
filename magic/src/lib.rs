pub use magic_derive::MagicModel;
use sqlx::SqlitePool;
#[derive(MagicModel)]
#[magic(table = "users")]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
}