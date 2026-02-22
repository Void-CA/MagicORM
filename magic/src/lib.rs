pub use magic_derive::MagicModel;

#[derive(MagicModel)]
#[magic(table = "users")]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_user_generates_new_struct() {
        let new_user = User::new(
            "Alice".to_string(),
            "alice@mail.com".to_string(),
        );

        // Si la macro generó NewUser correctamente,
        // esto debería compilar.
        let _name = new_user.name;
        let _email = new_user.email;
    }
}