mod create_database;
pub use create_database::*;
use diesel::prelude::*;

use crate::{
    errors::DomainError,
    models::{
        misc::{Job, JobStatus, NewJob},
        users::UserId,
    },
};

pub fn get_jobs(
    conn: &impl diesel::Connection<Backend = diesel::pg::Pg>,
) -> Result<Vec<Job>, DomainError> {
    use crate::schema::jobs::dsl as jobs;
    use crate::schema::users::dsl as users;
    Ok(jobs::jobs
        .inner_join(users::users)
        .select((
            jobs::id,
            jobs::job_id,
            users::username,
            jobs::status,
            jobs::status_message,
            jobs::created_at,
        ))
        .load::<Job>(conn)?)
}

pub fn get_jobs_by_user(
    user_id: &UserId,
    conn: &impl diesel::Connection<Backend = diesel::pg::Pg>,
) -> Result<Vec<Job>, DomainError> {
    use crate::schema::jobs::dsl as jobs;
    use crate::schema::users::dsl as users;
    Ok(jobs::jobs
        .inner_join(users::users)
        .select((
            jobs::id,
            jobs::job_id,
            users::username,
            jobs::status,
            jobs::status_message,
            jobs::created_at,
        ))
        .filter(users::id.eq(user_id))
        .load::<Job>(conn)?)
}

pub fn get_job_by_uuid(
    job_id: uuid::Uuid,
    conn: &impl diesel::Connection<Backend = diesel::pg::Pg>,
) -> Result<Option<Job>, DomainError> {
    use crate::schema::jobs::dsl as jobs;
    use crate::schema::users::dsl as users;
    Ok(jobs::jobs
        .inner_join(users::users)
        .select((
            jobs::id,
            jobs::job_id,
            users::username,
            jobs::status,
            jobs::status_message,
            jobs::created_at,
        ))
        .filter(jobs::job_id.eq(job_id))
        .first::<Job>(conn)
        .optional()?)
}

pub fn update_job_status(
    job_id: uuid::Uuid,
    new_status: JobStatus,
    status_message: Option<String>,
    conn: &impl diesel::Connection<Backend = diesel::pg::Pg>,
) -> Result<(), DomainError> {
    use crate::schema::jobs::dsl as jobs;
    diesel::update(jobs::jobs.filter(jobs::job_id.eq(job_id)))
        .set((
            jobs::status.eq(new_status),
            jobs::status_message.eq(status_message),
        ))
        .execute(conn)?;
    Ok(())
}

pub fn create_job(
    new_job: &NewJob,
    conn: &impl diesel::Connection<Backend = diesel::pg::Pg>,
) -> Result<Job, DomainError> {
    use crate::schema::jobs::dsl as jobs;
    let job = conn
        .transaction(|| {
            diesel::insert_into(jobs::jobs)
                .values(new_job)
                .execute(conn)?;

            get_job_by_uuid(new_job.job_id, conn)
        })?
        .ok_or_else(|| {
            DomainError::new_internal_error(
                "failed to retrieve created job".to_owned(),
            )
        })?;
    Ok(job)
}
