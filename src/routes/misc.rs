use actix_web::HttpResponse;

use crate::get_build_info;

pub async fn build_info_req() -> HttpResponse {
    HttpResponse::Ok().json(get_build_info())
}
