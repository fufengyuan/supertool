use serde::Serialize;

pub fn print_json<T: Serialize>(data: &T) {
    println!("{}", serde_json::to_string_pretty(data).unwrap_or_default());
}
pub fn print_error(msg: &str) {
    eprintln!("\x1b[31m✗\x1b[0m {}", msg);
}
pub fn print_success(msg: &str) {
    println!("\x1b[32m✓\x1b[0m {}", msg);
}
