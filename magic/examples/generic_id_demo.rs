//! Ejemplo demostrando la abstracción de ID genérico en magic_orm.
//! Este ejemplo muestra cómo el trait Model ahora soporta tipos de ID genéricos.
//!
//! Ejecutar con: cargo run --example generic_id_demo

use magic_orm::{prelude::*, register_models};

// Modelo 1: ID tradicional i64 (comportamiento default)
#[derive(MagicModel, Debug)]
#[magic(table = "users")]
pub struct User {
    pub id: i64,
    pub name: String,
    pub email: String,
}

// Modelo 2: ID tipo i32 (otro tipo numérico)
// Nota: Como el insert actual devuelve i64 (por SQLite last_insert_rowid),
// necesitamos insertar manualmente para tipos que no sean i64
#[derive(MagicModel, Debug)]
#[magic(table = "products")]
pub struct Product {
    pub id: i32,  // ID genérico - tipo i32
    pub name: String,
    pub price: f64,
}

register_models!(User, Product);

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Conectar a base de datos en memoria
    let pool = SqlitePool::connect("sqlite::memory:").await?;
    sqlx::query("PRAGMA foreign_keys = ON;")
        .execute(&pool)
        .await?;

    // Crear tablas
    create_all::<_, AppModels>(&pool).await?;

    println!("=== Demostración de ID Genérico ===\n");

    // 1. Usar modelo con ID i64 (comportamiento tradicional)
    println!("1. Trabajando con User (ID i64)...");
    let user_id = User::insert(&pool, &NewUser {
        name: "Alice".to_string(),
        email: "alice@example.com".to_string(),
    }).await?;
    println!("   Usuario creado con ID i64: {}", user_id);

    // Recuperar por ID (usa el tipo correcto automáticamente)
    let user = User::get_by_id(&pool, user_id).await?;
    println!("   Usuario recuperado: {:?}", user);

    // 2. Usar modelo con ID i32
    println!("\n2. Trabajando con Product (ID i32)...");
    
    // Insertar manualmente porque el método insert() devuelve i64
    let product_id = 100i32;
    sqlx::query("INSERT INTO products (id, name, price) VALUES (?, ?, ?)")
        .bind(product_id)
        .bind("Widget")
        .bind(29.99)
        .execute(&pool)
        .await?;
    println!("   Producto creado con ID i32: {}", product_id);

    // get_by_id ahora acepta i32 (el tipo de Product::Id)
    let product = Product::get_by_id(&pool, product_id).await?;
    println!("   Producto recuperado: {:?}", product);

    // 3. Delete con ID genérico
    println!("\n3. Eliminando con ID genérico...");
    let deleted = Product::delete_by_id(&pool, product_id).await?;
    println!("   Productos eliminados: {}", deleted);

    // 4. Verificar que las firmas aceptan el tipo correcto
    println!("\n4. Verificación de tipos:");
    println!("   User::get_by_id acepta: i64");
    println!("   Product::get_by_id acepta: i32");
    println!("   (Esto es generado automáticamente por el derive macro)");

    println!("\n=== Demo completada ===");
    println!("\nNota: La abstracción de ID permite que get_by_id y delete_by_id");
    println!("usen el tipo correcto según el modelo (i64 para User, i32 para Product).");
    println!("El método insert() todavía devuelve i64 debido a la limitación");
    println!("de SQLite (last_insert_rowid), pero las firmas de consulta son genéricas.");

    Ok(())
}
