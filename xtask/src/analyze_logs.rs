use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::{Map, Value};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{BufRead, BufReader};
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct LogLine {
    pub timestamp: DateTime<Utc>,
    pub level: String,
    pub target: String,
    #[serde(default)]
    pub fields: Fields,
    #[serde(default)]
    pub span: Option<SpanInfo>,
    #[serde(default)]
    pub spans: Vec<SpanInfo>,
}

#[derive(Debug, Default, Deserialize)]
pub struct Fields {
    #[serde(default)]
    pub message: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Deserialize)]
pub struct SpanInfo {
    pub name: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}
pub fn run(path: &str) -> anyhow::Result<()> {
    let file = OpenOptions::new().read(true).open(Path::new(path))?;
    let reader = BufReader::new(file);

    let mut level_counts: HashMap<String, usize> = HashMap::new();
    let mut span_busy_totals: HashMap<String, f64> = HashMap::new();
    let mut parse_errors = 0;
    let mut total_lines = 0;
    let now = std::time::Instant::now();

    for bytes in reader.split(b'\n') {
        total_lines += 1;
        let bytes = match bytes {
            Ok(b) => b,
            Err(e) => {
                println!("Error reading line: {e}");
                parse_errors += 1;
                continue;
            }
        };
        let line = String::from_utf8_lossy(&bytes)
            .to_string()
            .replace("�", "")
            .replace("\0", "");
        if line.trim().is_empty() {
            continue;
        }

        if total_lines % 100_000 == 0 {
            println!("Processed {} lines", total_lines);
        }

        let entry: LogLine = match serde_json::from_str(line.trim()) {
            Ok(e) => e,
            Err(err) => {
                println!("Error parsing line: {:?}", err);
                println!("Line: {:?} {}", line, line);
                parse_errors += 1;
                if parse_errors > 10 {
                    break;
                }
                continue;
            }
        };

        *level_counts.entry(entry.level.clone()).or_default() += 1;

        // "close" events carry time.busy for the span in `span`
        if entry.fields.message == "close"
            && let Some(span) = &entry.span
            && let Some(busy) = span.extra.get("time.busy").and_then(|v| v.as_str())
            && let Some(ms) = parse_duration_ms(busy)
        {
            *span_busy_totals.entry(span.name.clone()).or_default() += ms;
        }
    }

    println!("=== Parse time ===");
    println!("{:.2}s", now.elapsed().as_secs_f64());

    println!("=== Level counts ===");
    for (level, count) in &level_counts {
        println!("{level}: {count}");
    }

    println!("\n=== Total lines ===");
    println!("{total_lines}");

    println!("\n=== Total busy time by span (ms) ===");
    let mut totals: Vec<_> = span_busy_totals.into_iter().collect();
    totals.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    for (name, ms) in totals {
        println!("{name}: {ms:.2}ms");
    }

    if parse_errors > 0 {
        eprintln!("\nwarning: {parse_errors} lines failed to parse");
    }

    Ok(())
}

/// Parses tracing_subscriber's formatted durations like "26.5ms", "880µs", "2.23ms", "252s"
fn parse_duration_ms(s: &str) -> Option<f64> {
    let s = s.trim();
    let (num_part, unit) = if let Some(n) = s.strip_suffix("ms") {
        (n, 1.0)
    } else if let Some(n) = s.strip_suffix("µs") {
        (n, 0.001)
    } else if let Some(n) = s.strip_suffix("ns") {
        (n, 0.000_001)
    } else if let Some(n) = s.strip_suffix('s') {
        (n, 1000.0)
    } else {
        return None;
    };
    num_part.trim().parse::<f64>().ok().map(|v| v * unit)
}
