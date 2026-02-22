use magic_derive::MagicModel;

#[derive(MagicModel)]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
}