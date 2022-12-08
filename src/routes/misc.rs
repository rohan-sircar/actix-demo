use crate::get_build_info;

pub async fn build_info_req() -> String {
    serde_json::to_string(get_build_info()).unwrap()
}
