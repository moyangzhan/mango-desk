use crate::errors::AppError;
use crate::global::{DEFAULT_DATETIME_FORMAT, DEFAULT_DATETIME_MICRO_FORMAT};
use chrono::{DateTime, Local, NaiveDateTime};
use serde::{self, Deserialize, Deserializer, Serializer};

pub fn serialize<S>(date: &DateTime<Local>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let s = date.format(DEFAULT_DATETIME_FORMAT).to_string();
    serializer.serialize_str(&s)
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Local>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    str_to_datetime(s.as_str()).map_err(|e| serde::de::Error::custom(e.to_string()))
}

pub fn str_to_local_datetime(s: &str, fmt: &str) -> Result<DateTime<Local>, AppError> {
    if s.is_empty() {
        return Ok(DateTime::default());
    }
    NaiveDateTime::parse_from_str(s, fmt)
        .map_err(|e| AppError::DateTimeParseError(e.to_string()))
        .and_then(|naive| {
            naive
                .and_local_timezone(Local)
                .single()
                .ok_or(AppError::DateTimeParseError(
                    "Timezone conversion failed".to_string(),
                ))
        })
}

pub fn str_to_datetime(s: &str) -> Result<DateTime<Local>, AppError> {
    str_to_local_datetime(s, DEFAULT_DATETIME_FORMAT)
}

pub fn str_to_micro_datetime(s: &str) -> Result<DateTime<Local>, AppError> {
    str_to_local_datetime(s, DEFAULT_DATETIME_MICRO_FORMAT)
}

pub fn datetime_to_str(dt: &DateTime<Local>) -> String {
    dt.format(DEFAULT_DATETIME_FORMAT).to_string()
}

pub fn micro_datetime_to_str(dt: &DateTime<Local>) -> String {
    dt.format(DEFAULT_DATETIME_MICRO_FORMAT).to_string()
}
pub fn systemtime_to_datetime(time: std::time::SystemTime) -> DateTime<Local> {
    let datetime: DateTime<Local> = time.into();
    datetime
}

pub fn systemtime_to_str(time: std::time::SystemTime) -> String {
    let datetime: DateTime<Local> = systemtime_to_datetime(time);
    datetime_to_str(&datetime)
}

fn main() -> Result<(), AppError> {
    let dt = str_to_local_datetime("2024-05-20 14:30:00", "%Y-%m-%d %H:%M:%S")?;
    println!("local datetime: {}", dt);
    Ok(())
}
