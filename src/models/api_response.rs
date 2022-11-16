use serde::{Deserialize, Serialize};

#[derive(Eq, PartialEq, Debug, Clone, Serialize, Deserialize, new)]
pub struct ApiResponse<T> {
    success: bool,
    response: T,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn is_success(&self) -> bool {
        self.success
    }
    pub fn response(&self) -> &T {
        &self.response
    }
    pub fn successful(response: T) -> ApiResponse<T> {
        ApiResponse::new(true, response)
    }
    pub fn failure(response: T) -> ApiResponse<T> {
        ApiResponse::new(false, response)
    }
}
