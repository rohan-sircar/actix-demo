mod create_database;
use chrono::Duration;
pub use create_database::*;
use diesel::prelude::*;

use crate::{
    errors::DomainError,
    models::{
        misc::{Job, JobCount, JobStatus, NewJob},
        users::UserId,
    },
    types::DbConnection,
};

pub fn get_jobs(conn: &mut DbConnection) -> Result<Vec<Job>, DomainError> {
    use crate::schema::jobs::dsl as jobs;
    use crate::schema::users::dsl as users;
    Ok(jobs::jobs
        .inner_join(users::users)
        .select((
            jobs::id,
            jobs::job_id,
            users::id,
            jobs::status,
            jobs::status_message,
            jobs::created_at,
        ))
        .load::<Job>(conn)?)
}

pub fn get_jobs_by_user(
    user_id: &UserId,
    conn: &mut DbConnection,
) -> Result<Vec<Job>, DomainError> {
    use crate::schema::jobs::dsl as jobs;
    use crate::schema::users::dsl as users;
    Ok(jobs::jobs
        .inner_join(users::users)
        .select((
            jobs::id,
            jobs::job_id,
            users::id,
            jobs::status,
            jobs::status_message,
            jobs::created_at,
        ))
        .filter(users::id.eq(user_id))
        .load::<Job>(conn)?)
}

pub fn get_job_by_uuid(
    job_id: uuid::Uuid,
    conn: &mut DbConnection,
) -> Result<Option<Job>, DomainError> {
    use crate::schema::jobs::dsl as jobs;
    use crate::schema::users::dsl as users;
    Ok(jobs::jobs
        .inner_join(users::users)
        .select((
            jobs::id,
            jobs::job_id,
            users::id,
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
    conn: &mut DbConnection,
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
    conn: &mut DbConnection,
) -> Result<Job, DomainError> {
    use crate::schema::jobs::dsl as jobs;
    let job = conn
        .transaction(|conn| {
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

pub fn get_job_metrics(
    conn: &mut DbConnection,
    hours_since: Option<i8>,
    since_time: Option<chrono::NaiveDateTime>,
) -> Result<Vec<JobCount>, DomainError> {
    use crate::schema::jobs::dsl as jobs;
    use diesel::dsl::count;

    let mut query = jobs::jobs
        .group_by(jobs::status)
        .select((jobs::status, count(jobs::id)))
        .into_boxed();

    // Apply hours_since filter if provided
    if let Some(hours) = hours_since {
        let cutoff =
            chrono::Utc::now().naive_utc() - Duration::hours(hours as i64);
        query = query.filter(jobs::created_at.ge(cutoff));
    } else {
        // Apply specific timestamp filter if provided
        if let Some(time) = since_time {
            query = query.filter(jobs::created_at.ge(time));
        }
    }

    let res = query.load::<JobCount>(conn)?;
    Ok(res)
}
