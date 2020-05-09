use actix_web::{get, post, web, HttpResponse};

use crate::actions;
use crate::models;
use crate::types::DbPool;
use actix_web::error::ResponseError;
use std::rc::Rc;

/// Finds user by UID.
#[get("/api/authzd/users/get/{user_id}")]
pub async fn get_user(
    pool: web::Data<DbPool>,
    user_uid: web::Path<i32>,
) -> Result<HttpResponse, impl ResponseError> {
    let user_uid = user_uid.into_inner();
    // use web::block to offload blocking Diesel code without blocking server thread
    let res = web::block(move || {
        let conn = pool.get()?;
        actions::find_user_by_uid(user_uid, &conn)
    })
    .await
    .and_then(|maybe_user| {
        if let Some(user) = maybe_user {
            Ok(HttpResponse::Ok().json(user))
        } else {
            let res =
                HttpResponse::NotFound().body(format!("No user found with uid: {}", user_uid));
            Ok(res)
        }
    });
    res
}

#[get("/api/authzd/users/get")]
pub async fn get_all_users(pool: web::Data<DbPool>) -> Result<HttpResponse, impl ResponseError> {
    // use web::block to offload blocking Diesel code without blocking server thread
    let res = web::block(move || {
        let conn = pool.get()?;
        actions::get_all(&conn)
    })
    .await
    .and_then(|maybe_users| {
        debug!("{:?}", maybe_users);
        if let Some(users) = maybe_users {
            if users.is_empty() {
                let res = HttpResponse::Ok().json(models::ErrorModel {
                    status_code: 200,
                    reason: "No users available".to_string(),
                });
                Ok(res)
            } else {
                Ok(HttpResponse::Ok().json(users))
            }
        } else {
            let res = HttpResponse::Ok().json(models::ErrorModel {
                status_code: 200,
                reason: "No users available".to_string(),
            });
            Ok(res)
        }
    });
    res
}

/// Inserts new user with name defined in form.
#[post("/do_registration")]
pub async fn add_user(
    pool: web::Data<DbPool>,
    form: web::Json<models::NewUser>,
) -> Result<HttpResponse, impl ResponseError> {
    // use web::block to offload blocking Diesel code without blocking server thread
    let user = web::block(move || {
        let conn = pool.get()?;
        actions::insert_new_user(Rc::new(form.0), &conn)
    })
    .await
    .and_then(|user| {
        debug!("{:?}", user);
        Ok(HttpResponse::Created().json(user))
    });
    user
}
