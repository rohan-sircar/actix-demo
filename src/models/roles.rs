use crate::schema::roles;
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};

#[derive(DbEnum, Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
#[DieselType = "Role_name"]
pub enum RoleEnum {
    RoleSuperUser,
    RoleAdmin,
    RoleUser,
}

#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Identifiable)]
#[table_name = "roles"]
pub struct Role {
    pub id: i32,
    pub name: RoleEnum,
}
