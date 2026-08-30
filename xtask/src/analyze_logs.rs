use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use itertools::Itertools;

use crate::LogAnalysisCommand;
use crate::log_utils::{LogReader, SpanInfo};

pub fn run(
    path: &str,
    total_line_estimate: Option<u64>,
    command: LogAnalysisCommand,
) -> anyhow::Result<()> {
    let file = OpenOptions::new().read(true).open(Path::new(path))?;

    let log_reader = LogReader::new(file);

    match command {
        LogAnalysisCommand::ListErrors => list_errors(log_reader, total_line_estimate),
        LogAnalysisCommand::ListTopLevelSpans => {
            list_top_level_spans(log_reader, total_line_estimate)
        }
        LogAnalysisCommand::ListSqlx => list_sqlx(log_reader, total_line_estimate),
        LogAnalysisCommand::SplitTopLevelSpans { output_dir } => {
            split_top_level_spans(log_reader, total_line_estimate, &output_dir)?
        }
    }

    Ok(())
}

fn list_sqlx(log_reader: LogReader, total_line_estimate: Option<u64>) {
    let mut parse_errors = 0;
    let mut total_lines = 0;
    let now = std::time::Instant::now();

    for bytes_log_line_result in log_reader.iter() {
        total_lines += 1;
        if total_lines % 100_000 == 0 {
            let percent =
                (total_lines as f64 / total_line_estimate.unwrap_or(total_lines) as f64) * 100.0;
            println!("lines: {total_lines} {percent:.2}%");
        }
        match bytes_log_line_result {
            Ok(log_line) => {
                if log_line.target.starts_with("sqlx::") {
                    println!("sqlx line: {:?}", log_line);
                }
            }
            Err(e) => {
                eprintln!("warning: failed to parse line: {e}");
                parse_errors += 1;
            }
        }
    }

    println!("=== Parse time ===");
    println!("{:.2}s", now.elapsed().as_secs_f64());

    println!("\n=== Total lines ===");
    println!("{total_lines}");

    if parse_errors > 0 {
        eprintln!("\nwarning: {parse_errors} lines failed to parse");
    }
}

fn list_top_level_spans(log_reader: LogReader, total_line_estimate: Option<u64>) {
    let mut parse_errors = 0;
    let mut total_lines = 0;
    let now = std::time::Instant::now();

    for bytes_log_line_result in log_reader.iter() {
        total_lines += 1;
        if total_lines % 100_000 == 0 {
            let percent =
                (total_lines as f64 / total_line_estimate.unwrap_or(total_lines) as f64) * 100.0;
            println!("lines: {total_lines} {percent:.2}%");
        }
        match bytes_log_line_result {
            Ok(log_line) => {
                if log_line.spans.is_empty() && !(log_line.target.starts_with("sqlx::")) {
                    println!("Top Level span: {:?}", log_line);
                }
            }
            Err(e) => {
                eprintln!("warning: failed to parse line: {e}");
                parse_errors += 1;
            }
        }
    }

    println!("=== Parse time ===");
    println!("{:.2}s", now.elapsed().as_secs_f64());

    println!("\n=== Total lines ===");
    println!("{total_lines}");

    if parse_errors > 0 {
        eprintln!("\nwarning: {parse_errors} lines failed to parse");
    }
}

fn list_errors(log_reader: LogReader, total_line_estimate: Option<u64>) {
    let mut parse_errors = 0;
    let mut total_lines = 0;
    let now = std::time::Instant::now();

    for bytes_log_line_result in log_reader.iter() {
        total_lines += 1;
        if total_lines % 100_000 == 0 {
            let percent =
                (total_lines as f64 / total_line_estimate.unwrap_or(total_lines) as f64) * 100.0;
            println!("lines: {total_lines} {percent:.2}%");
        }
        match bytes_log_line_result {
            Ok(log_line) => {
                if log_line.level == "ERROR" {
                    println!("Error: {:?}", log_line);
                }
            }
            Err(e) => {
                eprintln!("warning: failed to parse line: {e}");
                parse_errors += 1;
            }
        }
    }

    println!("=== Parse time ===");
    println!("{:.2}s", now.elapsed().as_secs_f64());

    println!("\n=== Total lines ===");
    println!("{total_lines}");

    if parse_errors > 0 {
        eprintln!("\nwarning: {parse_errors} lines failed to parse");
    }
}

fn split_top_level_spans(
    log_reader: LogReader,
    total_line_estimate: Option<u64>,
    output_dir: &str,
) -> anyhow::Result<()> {
    let output_dir = PathBuf::from(output_dir);
    std::fs::create_dir_all(&output_dir)?;

    let mut parse_errors = 0u64;
    let mut total_lines = 0u64;
    let now = std::time::Instant::now();

    let mut top_level_span_writers: HashMap<Option<SpanInfo>, LogWriter> = HashMap::new();

    for line_result in log_reader.iter_raw() {
        total_lines += 1;
        if total_lines % 100_000 == 0 {
            let percent =
                (total_lines as f64 / total_line_estimate.unwrap_or(total_lines) as f64) * 100.0;
            println!("lines: {total_lines} {percent:.2}%");
        }
        match line_result {
            Ok((raw, log_line)) => {
                if log_line.target.starts_with("sqlx::") {
                    continue;
                }
                let top_level = log_line
                    .spans
                    .first()
                    .or_else(|| log_line.span.as_ref())
                    .cloned();
                let writer = top_level_span_writers
                    .entry(top_level.clone())
                    .or_insert_with(|| {
                        LogWriter::new(&output_dir.join(format!(
                        "{}.log",
                        top_level.map(|f| f.to_filename()).unwrap_or("root".to_string())
                    )))
                    });
                writer.write_line(&raw);
            }
            Err(e) => {
                eprintln!("warning: failed to parse line: {e}");
                parse_errors += 1;
            }
        }
    }

    println!("=== Parse time ===");
    println!("{:.2}s", now.elapsed().as_secs_f64());

    println!("\n=== Total lines ===");
    println!("{total_lines}");

    println!("\n=== Output ===");
    // todo add niche output
    println!("Total top level spans: {}", top_level_span_writers.len());
    println!("output directory: {}", output_dir.display());
    for (top_level, writer) in top_level_span_writers.into_iter().sorted_by(|a, b| {
        // a.0.as_ref()
        //     .map(|f| f.to_filename())
        //     .unwrap_or("root".to_string())
        //     .cmp(
        //         &b.0.as_ref()
        //             .map(|f| f.to_filename())
        //             .unwrap_or("root".to_string()),
        //     )
        //     .then_with(|| a.1.get_total_lines().cmp(&b.1.get_total_lines()))
        a.1.get_total_lines().cmp(&b.1.get_total_lines())
    }) {
        println!(
            "  {}: {}",
            top_level
                .as_ref()
                .map(|f| f.to_filename())
                .unwrap_or("root".to_string()),
            writer.get_total_lines()
        );
    }

    if parse_errors > 0 {
        println!("\n=== Warnings ===");
        println!("{parse_errors} lines failed to parse");
    }

    Ok(())
}

struct LogWriter {
    writer: std::fs::File,
    total_lines: u64,
}

impl LogWriter {
    pub fn new(path: &Path) -> Self {
        Self {
            writer: std::fs::File::create(path).unwrap(),
            total_lines: 0,
        }
    }

    pub fn write_line(&mut self, line: &str) {
        self.writer.write_all(line.as_bytes()).unwrap();
        self.writer.write_all(b"\n").unwrap();
        self.total_lines += 1;
    }

    pub fn get_total_lines(&self) -> u64 {
        self.total_lines
    }
}
