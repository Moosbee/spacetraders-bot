use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::{Map, Value};
use std::fs::File;
use std::io::{BufRead, BufReader};

#[derive(Debug, Deserialize, Clone, Hash, Eq, PartialEq)]
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

#[derive(Debug, Default, Deserialize, Clone, Hash, Eq, PartialEq)]
pub struct Fields {
    #[serde(default)]
    pub message: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Debug, Deserialize, Clone, Hash, Eq, PartialEq)]
pub struct SpanInfo {
    pub name: String,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

pub struct LogReader {
    reader: BufReader<File>,
}

impl LogReader {
    pub fn new(file: File) -> Self {
        Self {
            reader: BufReader::new(file),
        }
    }

    pub fn iter(self) -> impl Iterator<Item = Result<LogLine, std::io::Error>> {
        self.reader.split(b'\n').map(|line| {
            let bytes = line?;
            let line_str = String::from_utf8_lossy(&bytes)
                .to_string()
                .replace("�", "")
                .replace("\0", "");
            Ok(serde_json::from_str(line_str.trim())?)
        })
    }

    pub fn iter_raw(self) -> impl Iterator<Item = Result<(String, LogLine), std::io::Error>> {
        self.reader.split(b'\n').map(|line| {
            let bytes = line?;
            let line_str = String::from_utf8_lossy(&bytes)
                .to_string()
                .replace("�", "")
                .replace("\0", "");
            let log_line = serde_json::from_str(line_str.trim())?;
            Ok((line_str, log_line))
        })
    }
}

fn sanitize_filename(input: &str) -> String {
    let mut result = String::with_capacity(input.len());

    for c in input.chars() {
        match c {
            // Characters unsafe or inconvenient in filenames
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => {
                result.push('_');
            }

            // Whitespace becomes a readable separator
            c if c.is_whitespace() => {
                result.push('_');
            }

            // Control characters are removed
            c if c.is_control() => {}

            _ => result.push(c),
        }
    }

    // Collapse repeated underscores
    let mut cleaned = String::with_capacity(result.len());
    for c in result.chars() {
        if c == '_' && cleaned.ends_with('_') {
            continue;
        }
        cleaned.push(c);
    }

    // Filenames should not end in a space or dot
    cleaned.trim_end_matches([' ', '.']).to_string()
}

impl SpanInfo {
    pub fn to_filename(&self) -> String {
        let mut filename = sanitize_filename(&self.name);

        for key in ["ship", "socket_address", "self.ship_symbol"] {
            if let Some(Value::String(value)) = self.extra.get(key) {
                filename.push('_');
                filename.push_str(&sanitize_filename(key));
                filename.push('_');
                filename.push_str(&sanitize_filename(value));
            }
        }

        filename
    }
}
