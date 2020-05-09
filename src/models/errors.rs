use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct JsonErrorModel {
    status_code: i16,
    pub line: String,
    pub reason: String,
}
#[derive(Debug, Clone, Serialize)]
pub struct ErrorModel {
    pub status_code: i16,
    pub reason: String,
}
