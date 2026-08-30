use std::io;
use std::time::Instant;

use crate::log_utils::{LogLine, LogReader};

const PROGRESS_INTERVAL: u64 = 100_000;

/// Iterate over parsed log lines, printing progress and a final summary.
/// Returns the number of lines that failed to parse.
pub fn for_each_parsed_line<F>(
    log_reader: LogReader,
    total_line_estimate: Option<u64>,
    on_line: F,
) -> u64
where
    F: FnMut(LogLine),
{
    process_lines(log_reader.iter(), total_line_estimate, on_line)
}

/// Iterate over raw log lines (raw text plus parsed line), printing progress
/// and a final summary. Returns the number of lines that failed to parse.
pub fn for_each_raw_line<F>(
    log_reader: LogReader,
    total_line_estimate: Option<u64>,
    on_line: F,
) -> u64
where
    F: FnMut((String, LogLine)),
{
    process_lines(log_reader.iter_raw(), total_line_estimate, on_line)
}

pub fn print_parse_error_warning(parse_errors: u64) {
    if parse_errors > 0 {
        eprintln!("\nwarning: {parse_errors} lines failed to parse");
    }
}

fn process_lines<I, T, F>(lines: I, total_line_estimate: Option<u64>, mut on_line: F) -> u64
where
    I: Iterator<Item = io::Result<T>>,
    F: FnMut(T),
{
    let mut parse_errors = 0u64;
    let mut total_lines = 0u64;
    let now = Instant::now();

    for line_result in lines {
        total_lines += 1;
        print_progress(total_lines, total_line_estimate);
        match line_result {
            Ok(item) => on_line(item),
            Err(e) => {
                eprintln!("warning: failed to parse line: {e}");
                parse_errors += 1;
            }
        }
    }

    print_summary(now, total_lines);
    parse_errors
}

fn print_progress(total_lines: u64, total_line_estimate: Option<u64>) {
    if total_lines % PROGRESS_INTERVAL == 0 {
        let percent =
            (total_lines as f64 / total_line_estimate.unwrap_or(total_lines) as f64) * 100.0;
        println!("lines: {total_lines} {percent:.2}%");
    }
}

fn print_summary(now: Instant, total_lines: u64) {
    println!("=== Parse time ===");
    println!("{:.2}s", now.elapsed().as_secs_f64());

    println!("\n=== Total lines ===");
    println!("{total_lines}");
}
