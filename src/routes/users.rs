use actix_web::{web, HttpResponse};

use crate::actions;
use crate::models::misc::{Pagination, SearchQuery};
use crate::models::users::{NewUser, UserId};
use crate::{errors::DomainError, AppData};

/// Finds user by UID.
#[tracing::instrument(level = "info", skip(app_data))]
// #[has_any_role("RoleEnum::RoleAdmin", type = "RoleEnum")]
pub async fn get_user(
    app_data: web::Data<AppData>,
    user_id: web::Path<UserId>,
) -> Result<HttpResponse, DomainError> {
    let user_id = user_id.into_inner();
    let _ = tracing::info!("Getting user with id {user_id}");
    // use web::block to offload blocking Diesel code without blocking server thread
    let res = web::block(move || {
        let pool = &app_data.pool;
        let mut conn = pool.get()?;
        actions::users::find_user_by_uid(&user_id, &mut conn)
    })
    .await??;
    let _ = tracing::debug!("{:?}", res);
    if let Some(user) = res {
        let _ = tracing::info!("Found user");
        Ok(HttpResponse::Ok().json(user))
    } else {
        let _ = tracing::warn!("Could not find user");
        let err = DomainError::new_entity_does_not_exist_error(format!(
            "No user found with uid: {}",
            user_id
        ));
        Err(err)
    }
}

#[tracing::instrument(level = "info", skip(app_data))]
pub async fn get_users(
    app_data: web::Data<AppData>,
    pagination: web::Query<Pagination>,
) -> Result<HttpResponse, DomainError> {
    let _ = tracing::info!("Paginated users request");
    let users = web::block(move || {
        let pool = &app_data.pool;
        let mut conn = pool.get()?;
        let p: Pagination = pagination.into_inner();
        actions::users::get_all_users(&p, &mut conn)
    })
    .await??;

    let _ = tracing::info!("Found {} users", users.len());
    let _ = tracing::debug!("{:?}", users);

    Ok(HttpResponse::Ok().json(users))
}

#[tracing::instrument(level = "info", skip(app_data))]
pub async fn search_users(
    app_data: web::Data<AppData>,
    query: web::Query<SearchQuery>,
    pagination: web::Query<Pagination>,
) -> Result<HttpResponse, DomainError> {
    let _ = tracing::info!("Search users request");
    let users = web::block(move || {
        let pool = &app_data.pool;
        let mut conn = pool.get()?;
        let p: Pagination = pagination.into_inner();
        actions::users::search_users(query.q.as_str(), &p, &mut conn)
    })
    .await??;

    let _ = tracing::info!("Found {} users", users.len());
    let _ = tracing::debug!("{:?}", users);

    Ok(HttpResponse::Ok().json(users))
}

/// Inserts a new user
#[tracing::instrument(level = "info", skip(app_data))]
pub async fn add_user(
    app_data: web::Data<AppData>,
    form: web::Json<NewUser>,
) -> Result<HttpResponse, DomainError> {
    let user = web::block(move || {
        let pool = &app_data.pool;
        let mut conn = pool.get()?;
        actions::users::insert_new_regular_user(
            form.0,
            app_data.config.hash_cost,
            &mut conn,
        )
    })
    .await??;

    let _ = tracing::info!("Created user with id={}", user.id);
    let _ = tracing::debug!("{:?}", user);

    Ok(HttpResponse::Created().json(user))
}
