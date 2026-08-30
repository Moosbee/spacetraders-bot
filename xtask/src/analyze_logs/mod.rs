use std::fs::OpenOptions;
use std::path::Path;

use crate::LogAnalysisCommand;
use crate::log_utils::LogReader;

mod common;
mod list_errors;
mod list_spans;
mod span_stats;
mod split;

pub fn run(
    path: &str,
    total_line_estimate: Option<u64>,
    command: LogAnalysisCommand,
) -> anyhow::Result<()> {
    if let LogAnalysisCommand::ListSpanStats { dir } = &command {
        return span_stats::list_span_stats(dir);
    }

    let file = OpenOptions::new().read(true).open(Path::new(path))?;

    let log_reader = LogReader::new(file);

    match command {
        LogAnalysisCommand::ListErrors => list_errors::list_errors(log_reader, total_line_estimate),
        LogAnalysisCommand::ListTopLevelSpans => {
            list_spans::list_top_level_spans(log_reader, total_line_estimate)
        }
        LogAnalysisCommand::ListSqlx => list_spans::list_sqlx(log_reader, total_line_estimate),
        LogAnalysisCommand::SplitTopLevelSpans { output_dir } => {
            split::split_top_level_spans(log_reader, total_line_estimate, &output_dir)?
        }
        LogAnalysisCommand::ListSpanStats { .. } => unreachable!(),
    }

    Ok(())
}
