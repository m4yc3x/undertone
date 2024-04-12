use chrono::Local;
use colored::*;

pub fn log(message: &str) {
    let timestamp = Local::now();
    println!("[{}] {}", timestamp.format("%Y-%m-%d %H:%M:%S").to_string().yellow(), message.cyan());
}

pub fn err(message: &str) {
    let timestamp = Local::now();
    println!("[{}] [{}] {}", timestamp.format("%Y-%m-%d %H:%M:%S").to_string().yellow(), "ERROR".to_string().red(), message.cyan());
}
