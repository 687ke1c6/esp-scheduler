use chrono::Utc;
pub fn log(log_line: String) {
    println!("{} | {}", Utc::now().format("%d-%m-%Y %H:%M:%S"), log_line);
}