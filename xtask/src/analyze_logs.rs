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
    if let LogAnalysisCommand::ListSpanStats { dir } = &command {
        return list_span_stats(dir);
    }

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
        LogAnalysisCommand::ListSpanStats { .. } => unreachable!(),
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

#[derive(Default)]
struct SpanStats {
    count: u64,
    busy: Vec<f64>,
    idle: Vec<f64>,
}

fn list_span_stats(dir: &str) -> anyhow::Result<()> {
    let now = std::time::Instant::now();
    let dir_path = Path::new(dir);

    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir_path)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("log"))
        .collect();
    entries.sort();

    let mut stats: HashMap<String, SpanStats> = HashMap::new();
    let mut total_lines = 0u64;
    let mut close_events = 0u64;
    let mut parse_errors = 0u64;

    for path in &entries {
        let file = OpenOptions::new().read(true).open(path)?;
        let log_reader = LogReader::new(file);

        for line_result in log_reader.iter() {
            total_lines += 1;
            match line_result {
                Ok(log_line) => {
                    if log_line.fields.message == "close" {
                        close_events += 1;
                        let Some(span) = log_line.span.as_ref() else {
                            continue;
                        };
                        let entry = stats.entry(span.to_filename()).or_default();
                        entry.count += 1;
                        if let Some(busy) = log_line
                            .fields
                            .extra
                            .get("time.busy")
                            .and_then(|value| value.as_str())
                            .and_then(parse_duration)
                        {
                            entry.busy.push(busy);
                        }
                        if let Some(idle) = log_line
                            .fields
                            .extra
                            .get("time.idle")
                            .and_then(|value| value.as_str())
                            .and_then(parse_duration)
                        {
                            entry.idle.push(idle);
                        }
                    }
                }
                Err(_) => {
                    parse_errors += 1;
                }
            }
        }
    }

    let mut rows: Vec<(String, SpanStats)> = stats.into_iter().collect();
    rows.sort_by(|a, b| {
        let a_total: f64 = a.1.busy.iter().sum();
        let b_total: f64 = b.1.busy.iter().sum();
        b_total
            .partial_cmp(&a_total)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.0.cmp(&b.0))
    });

    println!("\n=== Span busy/idle stats ===");
    println!("files processed: {}", entries.len());
    println!("total lines: {total_lines}");
    println!("close events: {close_events}");
    println!("distinct spans: {}", rows.len());
    if parse_errors > 0 {
        println!("lines failed to parse: {parse_errors}");
    }

    for (name, span_stats) in rows {
        println!("\n{name} (n={})", span_stats.count);
        match summarize(&span_stats.busy) {
            Some((min, max, median, avg)) => println!(
                "  busy:  min {:>10}  max {:>10}  median {:>10}  avg {:>10}",
                format_duration(min),
                format_duration(max),
                format_duration(median),
                format_duration(avg)
            ),
            None => println!("  busy:  (no data)"),
        }
        match summarize(&span_stats.idle) {
            Some((min, max, median, avg)) => println!(
                "  idle:  min {:>10}  max {:>10}  median {:>10}  avg {:>10}",
                format_duration(min),
                format_duration(max),
                format_duration(median),
                format_duration(avg)
            ),
            None => println!("  idle:  (no data)"),
        }
    }

    println!("\n=== Parse time ===");
    println!("{:.2}s", now.elapsed().as_secs_f64());

    Ok(())
}

fn summarize(values: &[f64]) -> Option<(f64, f64, f64, f64)> {
    if values.is_empty() {
        return None;
    }

    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let min = sorted[0];
    let max = sorted[sorted.len() - 1];
    let median = if sorted.len() % 2 == 0 {
        (sorted[sorted.len() / 2 - 1] + sorted[sorted.len() / 2]) / 2.0
    } else {
        sorted[sorted.len() / 2]
    };
    let avg = values.iter().sum::<f64>() / values.len() as f64;

    Some((min, max, median, avg))
}

fn parse_duration(s: &str) -> Option<f64> {
    let s = s.trim();
    let split_at = s
        .find(|c: char| !c.is_ascii_digit() && c != '.' && c != '-')
        .unwrap_or(s.len());
    let (num, unit) = s.split_at(split_at);
    let value: f64 = num.parse().ok()?;
    let factor = match unit {
        "ns" => 1e-9,
        "µs" | "us" => 1e-6,
        "ms" => 1e-3,
        "s" => 1.0,
        "m" => 60.0,
        "h" => 3600.0,
        "d" => 86400.0,
        _ => return None,
    };
    Some(value * factor)
}

fn format_duration(secs: f64) -> String {
    if secs >= 86400.0 {
        format!("{:.2}d", secs / 86400.0)
    } else if secs >= 3600.0 {
        format!("{:.2}h", secs / 3600.0)
    } else if secs >= 60.0 {
        format!("{:.2}m", secs / 60.0)
    } else if secs >= 1.0 {
        format!("{:.3}s", secs)
    } else if secs >= 1e-3 {
        format!("{:.3}ms", secs * 1e3)
    } else if secs >= 1e-6 {
        format!("{:.3}us", secs * 1e6)
    } else {
        format!("{:.1}ns", secs * 1e9)
    }
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
