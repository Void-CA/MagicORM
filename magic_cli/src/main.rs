use clap::{Parser, Subcommand};

mod db;
mod migrate;

#[derive(Parser)]
#[command(name = "magic")]
#[command(about = "MagicORM CLI")]
struct Cli {
    #[arg(long, help = "Ruta de la base de datos")]
    db_path: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Inicializa la base de datos
    Db {
        #[command(subcommand)]
        action: DbAction,
    },
    /// Gestión de migraciones
    Migrate {
        #[command(subcommand)]
        action: MigrateAction,
    },
}

#[derive(Subcommand)]
enum DbAction {
    Init { path: Option<String> },
}

#[derive(Subcommand)]
enum MigrateAction {
    /// Crea una migración vacía
    New { name: String },
    /// Genera una migración desde los modelos (requiere setup adicional)
    Generate { name: String },
    /// Aplica migraciones pendientes
    Up,
    /// Revierte la última migración
    Down,
    /// Muestra el estado de las migraciones
    Status,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Db { action } => match action {
            DbAction::Init { path } => db::init(&path.or(cli.db_path.clone())).await.unwrap(),
        },
        Commands::Migrate { action } => {
            let db_path = cli.db_path.unwrap_or_else(|| "magic.db".to_string());
            match action {
                MigrateAction::New { name } => migrate::new(&name).unwrap(),
                MigrateAction::Generate { name } => migrate::generate(&name).await.unwrap(),
                MigrateAction::Up => migrate::up(&db_path).await.unwrap(),
                MigrateAction::Down => migrate::down(&db_path).await.unwrap(),
                MigrateAction::Status => migrate::status(&db_path).await.unwrap(),
            }
        }
    }
}
