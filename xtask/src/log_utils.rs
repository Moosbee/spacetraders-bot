use chrono::{DateTime, Utc};
use serde::Deserialize;
use serde_json::{Map, Value};
use std::fs::File;
use std::io::{BufRead, BufReader};

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
}
