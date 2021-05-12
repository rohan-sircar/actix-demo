use actix_web::{get, web, HttpResponse};

use crate::services::UserService;
use crate::{actions, models};
use crate::{errors::DomainError, AppData};
use actix_web::error::ResponseError;
use validator::Validate;

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
    user_id: web::Path<i32>,
) -> Result<HttpResponse, DomainError> {
    let u_id = user_id.into_inner();
    tracing::info!("Getting user with id {}", u_id);
    // use web::block to offload blocking Diesel code without blocking server thread
    let res = web::block(move || {
        let pool = &app_data.pool;
        let conn = pool.get()?;
        actions::find_user_by_uid(u_id, &conn)
    })
    .await
    .map_err(|err| DomainError::new_thread_pool_error(err.to_string()))?;
    tracing::trace!("{:?}", res);
    if let Some(user) = res {
        Ok(HttpResponse::Ok().json(user))
    } else {
        let err = DomainError::new_entity_does_not_exist_error(format!(
            "No user found with uid: {}",
            u_id
        ));
        Err(err)
    }
}

#[get("/get/users/{user_id}")]
pub async fn get_user2(
    user_service: web::Data<dyn UserService>,
    user_id: web::Path<i32>,
) -> Result<HttpResponse, DomainError> {
    let u_id = user_id.into_inner();
    let user = user_service.find_user_by_uid(u_id)?;
    if let Some(user) = user {
        Ok(HttpResponse::Ok().json(user))
    } else {
        let err = DomainError::new_entity_does_not_exist_error(format!(
            "No user found with uid: {}",
            u_id
        ));
        Err(err)
    }
}

///List all users
#[tracing::instrument(level = "debug", skip(app_data))]
pub async fn get_all_users(
    app_data: web::Data<AppData>,
) -> Result<HttpResponse, DomainError> {
    // use web::block to offload blocking Diesel code without blocking server thread
    let users = web::block(move || {
        let pool = &app_data.pool;
        let conn = pool.get()?;
        actions::get_all(&conn)
    })
    .await
    .map_err(|err| DomainError::new_thread_pool_error(err.to_string()))?;

    tracing::trace!("{:?}", users);

    if !users.is_empty() {
        Ok(HttpResponse::Ok().json(users))
    } else {
        Err(DomainError::new_entity_does_not_exist_error(
            "No users available".to_owned(),
        ))
    }
}
//TODO: Add refinement here
/// Inserts new user with name defined in form.
#[tracing::instrument(level = "debug", skip(app_data))]
pub async fn add_user(
    app_data: web::Data<AppData>,
    form: web::Json<models::NewUser>,
) -> Result<HttpResponse, impl ResponseError> {
    // use web::block to offload blocking Diesel code without blocking server thread
    let res = match form.0.validate() {
        Ok(_) => web::block(move || {
            let pool = &app_data.pool;
            let conn = pool.get()?;
            actions::insert_new_user(
                form.0,
                &conn,
                Some(app_data.config.hash_cost),
            )
        })
        .await
        .map(|user| {
            tracing::debug!("{:?}", user);
            HttpResponse::Created().json(user)
        }),

        Err(e) => {
            // let err = e.to_string();
            // web::block(move || {
            //     Err(crate::errors::DomainError::new_generic_error(err))
            // })
            // .await

            // let res2 =
            //     crate::errors::DomainError::new_generic_error(e.to_string());
            // Err(res2)
            // let res2 = crate::errors::DomainError::GenericError {
            //     cause: e.to_string(),
            // };
            // Err(res2)
            let res = HttpResponse::BadRequest().json(e);
            // .json(models::ErrorModel::new(
            //     40,
            //     "Error registering user due to validation errors",
            // ));
            Ok(res)
        }
    };

    res
}
