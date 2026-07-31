// scripts/src/main.rs
use clap::{Parser, Subcommand};

mod analyze_logs;

#[derive(Parser)]
#[command(name = "xtask")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    SayHello,
    AnalyzeLogs {
        #[arg(long, default_value = "log.txt")]
        file: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::SayHello => println!("Hello!"),
        Commands::AnalyzeLogs { file } => analyze_logs::run(&file)?,
    }

    Ok(())
}
