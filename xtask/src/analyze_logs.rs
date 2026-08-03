use std::fs::OpenOptions;
use std::path::Path;

use crate::log_utils::LogReader;

pub fn run(path: &str, total_line_estimate: Option<u64>) -> anyhow::Result<()> {
    let file = OpenOptions::new().read(true).open(Path::new(path))?;

    let log_reader = LogReader::new(file);

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
                // if log_line.level == "ERROR" {
                //     println!("Error: {:?}", log_line);
                // }
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

    Ok(())
}
