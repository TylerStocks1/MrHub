use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use serde_json::Value;

pub fn read_messages_from_file(file_path: &str) -> Vec<String> {
    let mut messages = Vec::new();

    if let Ok(lines) = read_lines(file_path) {
        for line in lines {
            if let Ok(line) = line {
                // Parse each line as JSON and add it to the list
                if let Ok(value) = serde_json::from_str::<Value>(&line) {
                    if let Some(content) = value.as_str() {
                        messages.push(content.to_string());
                    }
                }
            }
        }
    }

    messages
}

// Helper function to read lines from a file
fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}
