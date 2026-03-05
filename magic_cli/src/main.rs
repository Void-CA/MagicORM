use clap::{Parser, Subcommand};

mod db;
mod migrate;
mod model_loader;

#[derive(Parser)]
#[command(name = "magic")]
#[command(about = "CLI de MagicORM para manejo de DB y migraciones")]
struct Cli {
    /// Ruta de la base de datos (opcional)
    #[arg(long)]
    db_path: Option<String>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Db {
        #[command(subcommand)]
        action: DbAction,
    },
    // Migrate {
    //     #[command(subcommand)]
    //     action: MigrateAction,
    // },
    Status,
}

#[derive(Subcommand)]
enum DbAction {
    Init { path: Option<String> },
}

#[derive(Subcommand)]
enum MigrateAction {
    Generate,
    Up,
    Down,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Db { action } => match action {
            DbAction::Init { path } => db::init(&path.or(cli.db_path.clone())).await.unwrap(),
        },
        // Commands::Migrate { action } => match action {
        //     MigrateAction::Generate => migrate::generate(&cli.db_path).unwrap(),
        //     MigrateAction::Up => migrate::up(&cli.db_path).unwrap(),
        //     MigrateAction::Down => migrate::down(&cli.db_path).unwrap(),
        // },
        Commands::Status => migrate::status(&cli.db_path).await.unwrap(),
    }
}