use serde::Serialize;

#[derive(Debug, Clone, Serialize, new)]
pub struct JsonErrorModel<'a> {
    status_code: i16,
    pub line: String,
    pub reason: &'a str,
}
#[derive(Debug, Clone, Serialize, new)]
pub struct ErrorModel<'a> {
    // pub error_code: i16,
    pub success: bool,
    pub reason: &'a str,
}
