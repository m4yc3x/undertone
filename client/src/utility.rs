use chrono::Local;
use colored::*;

/// Logs a message to the console, prepended with a current timestamp.
/// `message`: &str - The message to be logged.
/// This function does not return any value.
pub fn log(message: &str) {
    let timestamp = Local::now();
    // Example: Print the timestamp in yellow and the message in green
    println!("[{}] {}", timestamp.format("%Y-%m-%d %H:%M:%S").to_string().yellow(), message.cyan());
}

pub fn err(message: &str) {
    let timestamp = Local::now();
    // Example: Print the timestamp in yellow and the message in green
    println!("[{}] [{}] {}", timestamp.format("%Y-%m-%d %H:%M:%S").to_string().yellow(), "ERROR".to_string().red(), message.cyan());
}
