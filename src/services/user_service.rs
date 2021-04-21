use std::rc::Rc;

use diesel::SqliteConnection;

use crate::{actions, errors, models, types::DbPool};

pub trait UserService {
    fn find_user_by_uid(
        &self,
        uid: i32,
    ) -> Result<Option<models::UserDTO>, errors::DomainError>;
    fn _find_user_by_name(
        &self,
        user_name: String,
    ) -> Result<Option<models::UserDTO>, errors::DomainError>;

    fn get_all(
        &self,
    ) -> Result<Option<Vec<models::UserDTO>>, errors::DomainError>;

    fn insert_new_user(
        &self,
        nu: models::NewUser,
    ) -> Result<models::UserDTO, errors::DomainError>;

    // fn woot(&self) -> i32;

    fn verify_password<'a>(
        &self,
        user_name: &'a String,
        given_password: &'a String,
    ) -> Result<bool, errors::DomainError>;
}

#[derive(Clone)]
pub struct UserServiceImpl {
    pub pool: DbPool,
}

impl UserService for UserServiceImpl {
    fn find_user_by_uid(
        &self,
        uid: i32,
    ) -> Result<Option<models::UserDTO>, errors::DomainError> {
        let conn = self.pool.get()?;
        actions::find_user_by_uid(uid, &conn)
    }

    fn _find_user_by_name(
        &self,
        user_name: String,
    ) -> Result<Option<models::UserDTO>, errors::DomainError> {
        let conn = self.pool.get()?;
        actions::_find_user_by_name(user_name, &conn)
    }

    fn get_all(
        &self,
    ) -> Result<Option<Vec<models::UserDTO>>, errors::DomainError> {
        let conn = self.pool.get()?;
        actions::get_all(&conn)
    }

    fn insert_new_user(
        &self,
        nu: models::NewUser,
    ) -> Result<models::UserDTO, errors::DomainError> {
        let conn = self.pool.get()?;
        actions::insert_new_user(nu, &conn)
    }

    fn verify_password<'b>(
        &self,
        user_name: &'b String,
        given_password: &'b String,
    ) -> Result<bool, errors::DomainError> {
        let conn = self.pool.get()?;
        actions::verify_password(user_name, given_password, &conn)
    }

    // async fn woot(&self) -> i32 {
    //     1
    // }
}
