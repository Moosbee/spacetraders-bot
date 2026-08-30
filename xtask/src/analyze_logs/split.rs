use std::collections::HashMap;
use std::io::Write;
use std::path::{Path, PathBuf};

use itertools::Itertools;

use crate::log_utils::{LogReader, SpanInfo};

use super::common;

pub fn split_top_level_spans(
    log_reader: LogReader,
    total_line_estimate: Option<u64>,
    output_dir: &str,
) -> anyhow::Result<()> {
    let output_dir = PathBuf::from(output_dir);
    std::fs::create_dir_all(&output_dir)?;

    let mut top_level_span_writers: HashMap<Option<SpanInfo>, LogWriter> = HashMap::new();

    let parse_errors =
        common::for_each_raw_line(log_reader, total_line_estimate, |(raw, log_line)| {
            if log_line.target.starts_with("sqlx::") {
                return;
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
                        top_level
                            .map(|f| f.to_filename())
                            .unwrap_or("root".to_string())
                    )))
                });
            writer.write_line(&raw);
        });

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
