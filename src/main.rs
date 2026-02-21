use async_trait::async_trait;
use sqlx::FromRow;

use crate::traits::model::Model;
pub mod executor;
pub mod traits;

#[derive(FromRow)]
struct User {
    id: i32,
    name: String,
    email: String,
}

#[async_trait]
impl Model for User {
    type Id = i32;

    fn table_name() -> &'static str {
        "users"
    }

    async fn find(id: Self::Id) -> Result<Self, sqlx::Error> {
        let pool = executor::get_pool();

        sqlx::query_as::<_, Self>(
            "SELECT * FROM users WHERE id = $1"
        )
        .bind(id)
        .fetch_one(pool)
        .await
    }

    async fn all() -> Result<Vec<Self>, sqlx::Error> {
        let pool = executor::get_pool();

        sqlx::query_as::<_, Self>(
            "SELECT * FROM users"
        )
        .fetch_all(pool)
        .await
    }

    async fn insert(&mut self) -> Result<(), sqlx::Error> {
        let pool = executor::get_pool();

        let row: Self = sqlx::query_as(
            "INSERT INTO users (name, email)
             VALUES ($1, $2)
             RETURNING *"
        )
        .bind(&self.name)
        .bind(&self.email)
        .fetch_one(pool)
        .await?;

        *self = row;

        Ok(())
    }

    async fn update(&self) -> Result<(), sqlx::Error> {
        let pool = executor::get_pool();

        sqlx::query(
            "UPDATE users
             SET name = $1, email = $2
             WHERE id = $3"
        )
        .bind(&self.name)
        .bind(&self.email)
        .bind(self.id)
        .execute(pool)
        .await?;

        Ok(())
    }

    async fn delete(&self) -> Result<(), sqlx::Error> {
        let pool = executor::get_pool();

        sqlx::query(
            "DELETE FROM users WHERE id = $1"
        )
        .bind(self.id)
        .execute(pool)
        .await?;

        Ok(())
    }
}

fn main() {
    println!("Hello, Magic ORM!");
}