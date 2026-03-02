use clap::{Parser, Subcommand};
use magic_orm::prelude::*;
use anyhow::Result;

#[derive(Parser)]
#[command(name = "magic_orm")]
#[command(about = "CLI para gestionar la base de datos de Magic ORM")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Crea todas las tablas en la base de datos
    CreateDb,
    /// Imprime el SQL de creación de tabla de un modelo
    ShowSql {
        #[arg(help = "Nombre del modelo, ejemplo: User")]
        model: String,
    },
    /// Resetea la base de datos (elimina todas las tablas)
    ResetDb,
}

pub async fn run() -> Result<()> {
    let cli = Cli::parse();

    // Conexión SQLite de ejemplo
    let pool = SqlitePool::connect("sqlite://test.db").await?;
    sqlx::query("PRAGMA foreign_keys = ON;").execute(&pool).await?;

    match cli.command {
        Commands::CreateDb => {
            create_all::<SqlitePool, AppModels>(&pool).await?;
            println!("Base de datos creada correctamente.");
        }
        Commands::ShowSql { model } => {
            let sql = match model.as_str() {
                "User" => create_table_sql::<User>(),
                "Post" => create_table_sql::<Post>(),
                "Reaction" => create_table_sql::<Reaction>(),
                _ => {
                    println!("Modelo desconocido: {}", model);
                    return Ok(());
                }
            };
            println!("{}", sql);
        }
        Commands::ResetDb => {
            // Para simplificar, eliminamos tablas explícitamente
            sqlx::query("DROP TABLE IF EXISTS reactions").execute(&pool).await?;
            sqlx::query("DROP TABLE IF EXISTS posts").execute(&pool).await?;
            sqlx::query("DROP TABLE IF EXISTS users").execute(&pool).await?;
            println!("Base de datos reseteada.");
        }
    }

    Ok(())
}