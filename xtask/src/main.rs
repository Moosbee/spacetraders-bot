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
    Export {
        #[command(subcommand)]
        command: ExportCommand,
    },
    #[command(alias = "gql", alias = "generate-graphql")]
    GenerateGraphQL,
}

#[derive(Subcommand)]
enum LogAnalysisCommand {
    ListErrors,
    ListTopLevelSpans,
    ListSqlx,
    /// Split the log into one file per top-level span
    SplitTopLevelSpans {
        /// Output directory for the split log files
        #[arg(long, default_value = "split_logs")]
        output_dir: String,
    },
}

#[derive(Subcommand)]
enum ExportCommand {
    /// Export all waypoints (optionally filtered by system)
    Waypoints { system: Option<String> },
    /// Export all systems
    Systems,
    /// Export jump gate connections
    JumpConnections,
    /// Export all ship routes
    Routes,
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
        Commands::Export { command } => match command {
            ExportCommand::Waypoints { system } => {
                exports::export_waypoints(system.as_deref()).await?
            }
            ExportCommand::Systems => exports::export_systems().await?,
            ExportCommand::JumpConnections => exports::export_jump_connections().await?,
            ExportCommand::Routes => exports::export_routes().await?,
        },
        Commands::GenerateGraphQL => exports::generate_graphql().await?,
    }

    Ok(())
}
