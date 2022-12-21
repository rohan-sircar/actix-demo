use crate::{get_build_info, utils};

pub async fn build_info_req() -> String {
    utils::jstr(get_build_info())
}
