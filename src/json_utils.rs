use tokio::fs::File;
use tokio::io::AsyncReadExt;
use serde_json::Value;
//couldnt tell you one thing this dose, just works :) thanks again reddit 
pub async fn read_messages_from_file(path: &str) -> String {
    let mut file = match File::open(path).await {
        Ok(f) => f,
        Err(_) => return String::new(),
    };

    let mut contents = String::new();
    if let Err(err) = file.read_to_string(&mut contents).await {
        eprintln!("Failed to read file: {}", err);
        return String::new();
    }

    // some json parsing 
    let json: Value = match serde_json::from_str(&contents) {
        Ok(v) => v,
        Err(_) => return String::new(),
    };

    // Ensure that the json is a single string
    json.as_str().unwrap_or("").to_string()
}

pub async fn write_messages_to_file(path: &str, messages: &str) {
    let json_data = serde_json::to_string(&messages).expect("Failed to serialize messages");

    if let Err(e) = tokio::fs::write(path, json_data).await {
        eprintln!("Failed to write to file: {}", e);
    }
}
