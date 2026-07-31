// scripts/src/main.rs
use clap::{Parser, Subcommand};

mod analyse_logs;

#[derive(Parser)]
#[command(name = "xtask")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    SayHello,
    AnalyseLogs {
        #[arg(long, default_value = "log.txt")]
        input: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::SayHello => println!("Hello!"),
        Commands::AnalyseLogs { input } => {
            println!("input: {}", input);
            let _ = analyse_logs::analyse_logs(input).await?;
        }
    }

    Ok(())
}
