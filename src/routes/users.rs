use actix_web::{get, post, web, HttpResponse};

use crate::errors::DomainError;
use crate::services::UserService;
use crate::AppConfig;
use crate::{actions, models};
use actix_web::error::ResponseError;
use validator::Validate;

/// Finds user by UID.
#[get("/get/users/{user_id}")]
pub async fn get_user(
    config: web::Data<AppConfig>,
    user_id: web::Path<i32>,
) -> Result<HttpResponse, DomainError> {
    let u_id = user_id.into_inner();
    // use web::block to offload blocking Diesel code without blocking server thread
    let res = web::block(move || {
        let pool = &config.pool;
        let conn = pool.get()?;
        actions::find_user_by_uid(u_id, &conn)
    })
    .await
    .map_err(|_err| {
        let res = DomainError::new_generic_error(format!(
            "No user found with uid: {}",
            u_id
        ));
        res
    })?;
    if let Some(user) = res {
        Ok(HttpResponse::Ok().json(user))
    } else {
        let res = HttpResponse::NotFound()
            .body(format!("No user found with uid: {}", u_id));
        Ok(res)
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
        let res = HttpResponse::NotFound()
            .body(format!("No user found with uid: {}", u_id));
        Ok(res)
    }
}

#[get("/get/users")]
pub async fn get_all_users(
    config: web::Data<AppConfig>,
) -> Result<HttpResponse, impl ResponseError> {
    // use web::block to offload blocking Diesel code without blocking server thread
    let res = web::block(move || {
        let pool = &config.pool;
        let conn = pool.get()?;
        actions::get_all(&conn)
    })
    .await
    .and_then(|maybe_users| {
        debug!("{:?}", maybe_users);
        if let Some(users) = maybe_users {
            if users.is_empty() {
                let res = HttpResponse::NotFound()
                    .json(models::ErrorModel::new(40, "No users available"));
                // let res = crate::errors::DomainError::new_generic_error("".to_owned());
                Ok(res)
            } else {
                Ok(HttpResponse::Ok().json(users))
            }
        } else {
            let res = HttpResponse::NotFound()
                .json(models::ErrorModel::new(40, "No users available"));
            Ok(res)
        }
    });
    res
}

/// Inserts new user with name defined in form.
#[post("/do_registration")]
pub async fn add_user(
    config: web::Data<AppConfig>,
    form: web::Json<models::NewUser>,
) -> Result<HttpResponse, impl ResponseError> {
    // use web::block to offload blocking Diesel code without blocking server thread
    let res = match form.0.validate() {
        Ok(_) => web::block(move || {
            let pool = &config.pool;
            let conn = pool.get()?;
            actions::insert_new_user(form.0, &conn)
        })
        .await
        .and_then(|user| {
            debug!("{:?}", user);
            Ok(HttpResponse::Created().json(user))
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
            let res = HttpResponse::BadRequest().body(e.to_string());
            // .json(models::ErrorModel::new(
            //     40,
            //     "Error registering user due to validation errors",
            // ));
            Ok(res)
        }
    };

    res
}
