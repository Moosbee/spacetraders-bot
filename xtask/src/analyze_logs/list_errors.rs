use crate::log_utils::LogReader;

use super::common;

pub fn list_errors(log_reader: LogReader, total_line_estimate: Option<u64>) {
    let parse_errors = common::for_each_parsed_line(log_reader, total_line_estimate, |log_line| {
        if log_line.level == "ERROR" {
            println!("Error: {:?}", log_line);
        }
    });

    common::print_parse_error_warning(parse_errors);
}
