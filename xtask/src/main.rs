// scripts/src/main.rs
use clap::{Parser, Subcommand};

mod analyze_logs;
mod exports;
mod log_utils;

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

        #[arg(long)]
        total_lines: Option<u64>,

        #[command(subcommand)]
        commands: LogAnalysisCommand,
    },
    ExportSystemWaypoints {
        system: String,
    },
}

#[derive(Subcommand)]
enum LogAnalysisCommand {
    ListErrors,
    ListTopLevelSpans,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let _erg = dotenvy::dotenv();

    match cli.command {
        Commands::SayHello => println!("Hello!"),
        Commands::AnalyzeLogs {
            file,
            total_lines,
            commands,
        } => analyze_logs::run(&file, total_lines, commands)?,
        Commands::ExportSystemWaypoints { system } => {
            exports::export_system_waypoints(&system).await?
        }
    }

    Ok(())
}
