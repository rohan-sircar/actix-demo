use actix_web::{post, web, HttpResponse};

use crate::{
    actions,
    models::{self, ApiResponse, Pagination, SearchQuery, UserId},
};
use crate::{errors::DomainError, AppData};

/// Finds user by UID.
#[tracing::instrument(
    level = "debug",
    skip(app_data),
    fields(
        user_id = %user_id.0
    )
)]
pub async fn get_user(
    app_data: web::Data<AppData>,
    user_id: web::Path<UserId>,
) -> Result<HttpResponse, DomainError> {
    let u_id = user_id.into_inner();
    let u_id2 = u_id.clone();
    let _ = tracing::info!("Getting user with id {}", u_id);
    // use web::block to offload blocking Diesel code without blocking server thread
    let res = web::block(move || {
        let pool = &app_data.pool;
        let conn = pool.get()?;
        actions::find_user_by_uid(&u_id2, &conn)
    })
    .await
    .map_err(|err| DomainError::new_thread_pool_error(err.to_string()))??;
    let _ = tracing::trace!("{:?}", res);
    if let Some(user) = res {
        Ok(HttpResponse::Ok().json(ApiResponse::successful(user)))
    } else {
        let err = DomainError::new_entity_does_not_exist_error(format!(
            "No user found with uid: {}",
            u_id
        ));
        Err(err)
    }
}

// #[get("/get/users/{user_id}")]
// pub async fn get_user2(
//     user_service: web::Data<dyn UserService>,
//     user_id: web::Path<i32>,
// ) -> Result<HttpResponse, DomainError> {
//     let u_id = user_id.into_inner();
//     let user = user_service.find_user_by_uid(u_id)?;
//     if let Some(user) = user {
//         Ok(HttpResponse::Ok().json(user))
//     } else {
//         let err = DomainError::new_entity_does_not_exist_error(format!(
//             "No user found with uid: {}",
//             u_id
//         ));
//         Err(err)
//     }
// }

#[tracing::instrument(level = "debug", skip(app_data))]
pub async fn get_users(
    app_data: web::Data<AppData>,
    pagination: web::Query<Pagination>,
) -> Result<HttpResponse, DomainError> {
    let _ = tracing::info!("Paginated users request");
    let users = web::block(move || {
        let pool = &app_data.pool;
        let conn = pool.get()?;
        let p: Pagination = pagination.into_inner();
        actions::get_all_users(&p, &conn)
    })
    .await
    .map_err(|err| DomainError::new_thread_pool_error(err.to_string()))??;

    let _ = tracing::trace!("{:?}", users);

    Ok(HttpResponse::Ok().json(ApiResponse::successful(users)))
}

#[tracing::instrument(level = "debug", skip(app_data))]
pub async fn search_users(
    app_data: web::Data<AppData>,
    query: web::Query<SearchQuery>,
    pagination: web::Query<Pagination>,
) -> Result<HttpResponse, DomainError> {
    let _ = tracing::info!("Search users request");
    let users = web::block(move || {
        let pool = &app_data.pool;
        let conn = pool.get()?;
        let p: Pagination = pagination.into_inner();
        actions::search_users(query.q.as_str(), &p, &conn)
    })
    .await
    .map_err(|err| DomainError::new_thread_pool_error(err.to_string()))??;

    let _ = tracing::trace!("{:?}", users);

    Ok(HttpResponse::Ok().json(ApiResponse::successful(users)))
}

/// Inserts a new user
#[post("/api/registration")]
#[tracing::instrument(level = "debug", skip(app_data))]
pub async fn add_user(
    app_data: web::Data<AppData>,
    form: web::Json<models::NewUser>,
) -> Result<HttpResponse, DomainError> {
    let user = web::block(move || {
        let pool = &app_data.pool;
        let conn = pool.get()?;
        actions::insert_new_user(form.0, &conn, Some(app_data.config.hash_cost))
    })
    .await
    .map_err(|err| DomainError::new_thread_pool_error(err.to_string()))??;

    let _ = tracing::trace!("{:?}", user);

    Ok(HttpResponse::Created().json(ApiResponse::successful(user)))
}
