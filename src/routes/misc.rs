use actix_web::HttpResponse;

use crate::get_build_info;

#[utoipa::path(
    get,
    path = "/api/public/build-info",
    tag = "public",
    responses(
        (status = 200, description = "Build information"),
    ),
)]
pub async fn build_info_req() -> HttpResponse {
    HttpResponse::Ok().json(get_build_info())
}
