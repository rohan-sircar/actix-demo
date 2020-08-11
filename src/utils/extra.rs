#[derive(Debug, Serialize, Deserialize)]
struct MyObj {
    name: String,
    // number: i32,
}

#[get("/{id}/{name}")]
async fn index(info: web::Path<(u32, String)>) -> Result<HttpResponse, Error> {
    let (id, name) = (info.0, info.1.clone());
    let template = models::CardTemplate {
        title: "My Title",
        body: name,
        num: id,
    };
    template
        .call()
        .map(|body| HttpResponse::Ok().content_type("text/html").body(body))
        .map_err(|_| {
            error::ErrorInternalServerError("Error while parsing template")
        })
}

/// This handler uses json extractor
#[post("/extractor")]
async fn extract_my_obj(item: web::Json<MyObj>) -> HttpResponse {
    debug!("model: {:?}", item);
    HttpResponse::Ok().json(item.0) // <- send response
}