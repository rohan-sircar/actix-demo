use crate::schema::jobs;
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};

use super::users::{UserId, Username};

#[derive(Eq, PartialEq, Debug, Clone, Serialize, Deserialize, new)]
pub struct ErrorResponse<T> {
    pub cause: T,
}

impl<T: Serialize> ErrorResponse<T> {
    pub fn failure(response: T) -> ErrorResponse<T> {
        ErrorResponse::new(response)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(try_from = "u16")]
pub struct PaginationOffset(u16);
impl PaginationOffset {
    pub fn as_uint(&self) -> u16 {
        self.0
    }
}

impl TryFrom<u16> for PaginationOffset {
    type Error = String;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if value <= 2500 {
            Ok(PaginationOffset(value))
        } else {
            Err("Failed to validate".to_owned())
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(try_from = "u16")]
pub struct PaginationLimit(u16);
impl PaginationLimit {
    pub fn as_uint(&self) -> u16 {
        self.0
    }
}

impl TryFrom<u16> for PaginationLimit {
    type Error = String;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if value <= 50 {
            Ok(PaginationLimit(value))
        } else {
            Err("Failed to validate".to_owned())
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(try_from = "u16")]
pub struct PaginationPage(u16);
impl PaginationPage {
    pub fn as_uint(&self) -> u16 {
        self.0
    }
}

impl TryFrom<u16> for PaginationPage {
    type Error = String;
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if value <= 50 {
            Ok(PaginationPage(value))
        } else {
            Err("Failed to validate".to_owned())
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Pagination {
    pub page: PaginationPage,
    pub limit: PaginationLimit,
}

impl Pagination {
    pub fn calc_offset(&self) -> PaginationOffset {
        let res = self.page.as_uint() * self.limit.as_uint();
        PaginationOffset::try_from(res).unwrap()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchQueryString(String);

impl SearchQueryString {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchQuery {
    pub q: SearchQueryString,
    // pub pagination: Pagination
}

#[derive(DbEnum, Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
#[serde(rename_all = "snake_case")]
// #[DieselType = "Job_status"]
#[ExistingTypePath = "crate::schema::sql_types::JobStatus"]
pub enum JobStatus {
    Pending,
    Completed,
    Aborted,
    Failed,
}

#[derive(Debug, Clone, Deserialize, Serialize, Queryable, Identifiable)]
#[diesel(table_name = jobs)]
pub struct Job {
    pub id: i32,
    pub job_id: uuid::Uuid,
    pub started_by: Username,
    pub status: JobStatus,
    pub status_message: Option<String>,
    pub created_at: chrono::NaiveDateTime,
}

#[derive(Debug, Clone, Deserialize, Serialize, Insertable)]
#[diesel(table_name = jobs)]
pub struct NewJob {
    pub job_id: uuid::Uuid,
    pub started_by: UserId,
    pub status: JobStatus,
    pub status_message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pagination_refinement_test() {
        let mb_pag =
            serde_json::from_str::<Pagination>(r#"{"limit":5,"page":5}"#);
        // println!("{:?}", mb_pag);
        assert!(mb_pag.is_ok());
        let mb_pag =
            serde_json::from_str::<Pagination>(r#"{"limit":51,"page":5}"#);
        assert!(mb_pag.is_err());
        let mb_pag =
            serde_json::from_str::<Pagination>(r#"{"limit":5,"page":51}"#);
        assert!(mb_pag.is_err());
    }
}
