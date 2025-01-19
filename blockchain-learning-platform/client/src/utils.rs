// client/src/utils.rs

use chrono::{Utc, TimeZone};

/// 현재 UTC 시각을 RFC3339(ISO8601) 문자열로 반환
pub fn current_timestamp() -> String {
    Utc::now().to_rfc3339()
}

/// 심플 로그 함수
pub fn log_info(msg: &str) {
    println!("[INFO] {}", msg);
}

/// 에러 로그 함수
pub fn log_error(msg: &str) {
    eprintln!("[ERROR] {}", msg);
}
