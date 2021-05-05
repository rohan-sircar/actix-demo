use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, new)]
pub struct JsonErrorModel<'a> {
    status_code: i16,
    pub line: String,
    pub reason: &'a str,
}
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, new)]
pub struct ErrorModel {
    // pub error_code: i16,
    pub success: bool,
    pub reason: String,
}
