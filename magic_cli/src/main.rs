use anyhow::Ok;

mod cli;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("¡Bienvenido a Magic CLI!");
    Ok(())
}