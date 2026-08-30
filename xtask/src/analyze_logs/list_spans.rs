use crate::log_utils::LogReader;

use super::common;

pub fn list_sqlx(log_reader: LogReader, total_line_estimate: Option<u64>) {
    let parse_errors = common::for_each_parsed_line(log_reader, total_line_estimate, |log_line| {
        if log_line.target.starts_with("sqlx::") {
            println!("sqlx line: {:?}", log_line);
        }
    });

    common::print_parse_error_warning(parse_errors);
}

pub fn list_top_level_spans(log_reader: LogReader, total_line_estimate: Option<u64>) {
    let parse_errors = common::for_each_parsed_line(log_reader, total_line_estimate, |log_line| {
        if log_line.spans.is_empty() && !log_line.target.starts_with("sqlx::") {
            println!("Top Level span: {:?}", log_line);
        }
    });

    common::print_parse_error_warning(parse_errors);
}
