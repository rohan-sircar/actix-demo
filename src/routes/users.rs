use actix_web::{get, post, web, Error, HttpResponse};

use crate::actions;
use crate::models;
use crate::types::DbPool;

/// Finds user by UID.
#[get("/api/authzd/users/get/{user_id}")]
pub async fn get_user(
    pool: web::Data<DbPool>,
    user_uid: web::Path<i32>,
) -> Result<HttpResponse, Error> {
    let user_uid = user_uid.into_inner();
    // use web::block to offload blocking Diesel code without blocking server thread
    let maybe_user = web::block(move || {
        let conn = pool.get().map_err(|e| e.to_string())?;
        actions::find_user_by_uid(user_uid.into(), &conn).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| {
        error!("{}", e);
        HttpResponse::InternalServerError().finish()
    })?;

    if let Some(user) = maybe_user {
        Ok(HttpResponse::Ok().json(user))
    } else {
        let res = HttpResponse::NotFound().body(format!("No user found with uid: {}", user_uid));
        Ok(res)
    }
}

#[get("/api/authzd/users/get")]
pub async fn get_all_users(pool: web::Data<DbPool>) -> Result<HttpResponse, Error> {
    // use web::block to offload blocking Diesel code without blocking server thread
    let maybe_users = web::block(move || {
        let conn = pool.get().map_err(|e| e.to_string())?;
        actions::get_all(&conn).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| {
        eprintln!("{}", e);
        HttpResponse::InternalServerError().finish()
    })?;

    if let Some(users) = maybe_users {
        Ok(HttpResponse::Ok().json(users))
    } else {
        let res = HttpResponse::NotFound().body(format!("No users available"));
        Ok(res)
    }
    // Ok(HttpResponse::Ok().json(users))
}

/// Inserts new user with name defined in form.
#[post("/api/authzd/users/post")]
pub async fn add_user(
    pool: web::Data<DbPool>,
    form: web::Json<models::NewUser>,
) -> Result<HttpResponse, Error> {
    // use web::block to offload blocking Diesel code without blocking server thread
    let user = web::block(move || {
        let conn = pool.get().map_err(|e| e.to_string())?;
        actions::insert_new_user(&form, &conn).map_err(|e| e.to_string())
    })
    .await
    .map(|user| {
        debug!("{:?}", user);
        Ok(HttpResponse::Ok().json(user))
    })
    .map_err(|e| {
        eprintln!("{}", e);
        HttpResponse::InternalServerError().finish()
    })?;
    user
}
