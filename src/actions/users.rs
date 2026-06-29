use diesel::prelude::*;

use crate::errors::DomainError;
use crate::models::misc::Pagination;
use crate::models::roles::{NewUserRole, RoleEnum, RoleId};
use crate::models::users::{
    CreateProfile, Email, NewUser, OAuthProvider, OAuthUserLookup, Password,
    Profile, PublicProfile, UpdateProfile, UpdateUserProfile, User,
    UserAuthDetails, UserAuthDetailsWithRoles, UserId, UserUuid, UserWithRoles,
    Username,
};
use crate::types::DbConnection;
use crate::utils::InstrumentedRedisCache;
use bcrypt::hash;
use do_notation::m;
use validators::prelude::*;

/// Resolves a UserUuid to a UserId (integer ID).
pub fn resolve_user_id_by_uuid(
    uuid: &UserUuid,
    conn: &mut DbConnection,
) -> Result<Option<UserId>, DomainError> {
    use crate::schema::users::dsl as users;
    let user_id = users::users
        .select(users::id)
        .filter(users::user_uuid.eq(uuid))
        .first::<UserId>(conn)
        .optional()?;
    Ok(user_id)
}

/// Resolves a UserUuid to a UserId, returning an entity not found error if not found.
pub fn get_user_id_by_uuid(
    uuid: &UserUuid,
    conn: &mut DbConnection,
) -> Result<UserId, DomainError> {
    resolve_user_id_by_uuid(uuid, conn)?.ok_or_else(|| {
        DomainError::new_entity_does_not_exist_error(
            "User not found".to_string(),
        )
    })
}

pub fn get_roles_for_user(
    uid: &UserId,
    conn: &mut DbConnection,
) -> Result<Vec<RoleEnum>, DomainError> {
    use crate::schema::roles::dsl as roles;
    use crate::schema::users_roles::dsl as users_roles;
    Ok(users_roles::users_roles
        .inner_join(roles::roles)
        .select(roles::role_name)
        .filter(users_roles::user_id.eq(uid))
        .load::<RoleEnum>(conn)?)
}

pub fn get_roles_for_users(
    users: Vec<User>,
    conn: &mut DbConnection,
) -> Result<Vec<(UserId, UserWithRoles)>, DomainError> {
    users
        .into_iter()
        .map(|user| {
            let id = user.id;
            get_roles_for_user(&id, conn)
                .map(|roles| (id, UserWithRoles::from_user(&user, &roles)))
        })
        .collect::<Result<Vec<(UserId, UserWithRoles)>, DomainError>>()
}

pub fn find_user_by_uuid(
    uuid: &UserUuid,
    conn: &mut DbConnection,
) -> Result<Option<(UserId, UserWithRoles)>, DomainError> {
    use crate::schema::users::dsl as users;

    conn.transaction(|conn| {
        let mb_user = users::users
            .select((
                users::id,
                users::username,
                users::created_at,
                users::deleted_at,
                users::user_uuid,
                users::email_verified,
            ))
            .filter(users::user_uuid.eq(uuid))
            .first::<User>(conn)
            .optional()?;

        let roles = mb_user
            .as_ref()
            .map(|u| get_roles_for_user(&u.id, conn))
            .transpose()?;

        Ok(mb_user.map(|user| {
            (
                user.id,
                UserWithRoles::from_user(&user, &roles.unwrap_or_default()),
            )
        }))
    })
}

/// Like `find_user_by_uuid` but excludes soft-deleted users.
pub fn find_active_user_by_uuid(
    uuid: &UserUuid,
    conn: &mut DbConnection,
) -> Result<Option<(UserId, UserWithRoles)>, DomainError> {
    use crate::schema::users::dsl as users;

    conn.transaction(|conn| {
        let mb_user = users::users
            .select((
                users::id,
                users::username,
                users::created_at,
                users::deleted_at,
                users::user_uuid,
                users::email_verified,
            ))
            .filter(users::user_uuid.eq(uuid))
            .filter(users::deleted_at.is_null())
            .first::<User>(conn)
            .optional()?;

        let roles = mb_user
            .as_ref()
            .map(|u| get_roles_for_user(&u.id, conn))
            .transpose()?;

        Ok(mb_user.map(|user| {
            (
                user.id,
                UserWithRoles::from_user(&user, &roles.unwrap_or_default()),
            )
        }))
    })
}

pub fn find_user_by_name(
    user_name: &Username,
    conn: &mut DbConnection,
) -> Result<Option<(UserId, UserWithRoles)>, DomainError> {
    use crate::schema::users::dsl as users;

    conn.transaction(|conn| {
        let mb_user = users::users
            .select((
                users::id,
                users::username,
                users::created_at,
                users::deleted_at,
                users::user_uuid,
                users::email_verified,
            ))
            .filter(users::username.eq(user_name))
            .filter(users::deleted_at.is_null())
            .first::<User>(conn)
            .optional()?;

        let roles = match &mb_user {
            Some(user) => Some(get_roles_for_user(&user.id, conn)?),
            None => None,
        };

        let mb_user_with_roles = m! {
            user <- mb_user;
            roles <- roles;
            Some((user.id, UserWithRoles {
                username: user.username,
                created_at: user.created_at,
                user_uuid: user.user_uuid,
                roles,
            }))
        };

        Ok(mb_user_with_roles)
    })
}

pub fn get_user_auth_details(
    user_name: &Username,
    conn: &mut DbConnection,
) -> Result<Option<UserAuthDetailsWithRoles>, DomainError> {
    use crate::schema::users::dsl as users;

    conn.transaction(|conn| {
        let mb_user = users::users
            .select((
                users::id,
                users::username,
                users::password,
                users::email,
                users::oauth_provider,
                users::oauth_uid,
                users::user_uuid,
                users::email_verified,
            ))
            .filter(users::username.eq(user_name))
            .filter(users::deleted_at.is_null())
            .first::<UserAuthDetails>(conn)
            .optional()?;

        let roles = match &mb_user {
            Some(user) => Some(get_roles_for_user(&user.id, conn)?),
            None => None,
        };

        let mb_user_with_roles = m! {
            user <- mb_user;
            roles <- roles;
            Some(UserAuthDetailsWithRoles::from_user(user, roles))
        };

        Ok(mb_user_with_roles)
    })
}

pub fn get_all_users(
    pagination: &Pagination,
    conn: &mut DbConnection,
) -> Result<Vec<(UserId, UserWithRoles)>, DomainError> {
    use crate::schema::users::dsl as users;

    conn.transaction(|conn| {
        let users = users::users
            .select((
                users::id,
                users::username,
                users::created_at,
                users::deleted_at,
                users::user_uuid,
                users::email_verified,
            ))
            .filter(users::deleted_at.is_null())
            .order_by(users::created_at)
            .offset(pagination.calc_offset().as_uint().into())
            .limit(pagination.limit.as_uint().into())
            .load::<User>(conn)?;

        get_roles_for_users(users, conn)
    })
}

pub fn get_all_user_ids(
    cache: &InstrumentedRedisCache<String, Vec<UserId>>,
    conn: &mut DbConnection,
) -> Result<Vec<UserId>, DomainError> {
    use crate::schema::users::dsl as users;

    if let Ok(Some(cached)) = cache.get(&"user_ids".to_owned()) {
        tracing::debug!("cache size: {}", cached.len());
        tracing::trace!("cache: {:?}", cached);
        Ok(cached)
    } else {
        let users = users::users
            .select(users::id)
            .filter(users::deleted_at.is_null())
            .order_by(users::created_at)
            .load::<UserId>(conn)?;

        cache
            .set("user_ids".to_owned(), users.clone())
            .map_err(|e| {
                DomainError::new_internal_error(format!(
                    "Failed to set cache: {e:?}"
                ))
            })?;

        Ok(users)
    }
}

pub fn get_all_user_uuids(
    conn: &mut DbConnection,
) -> Result<Vec<UserUuid>, DomainError> {
    use crate::schema::users::dsl as users;

    let user_uuids = users::users
        .select(users::user_uuid)
        .filter(users::deleted_at.is_null())
        .order_by(users::created_at)
        .load::<UserUuid>(conn)?;

    Ok(user_uuids)
}

pub fn search_users(
    query: &str,
    pagination: &Pagination,
    conn: &mut DbConnection,
) -> Result<Vec<(UserId, UserWithRoles)>, DomainError> {
    use crate::schema::users::dsl as users;

    conn.transaction(|conn| {
        let users = users::users
            .select((
                users::id,
                users::username,
                users::created_at,
                users::deleted_at,
                users::user_uuid,
                users::email_verified,
            ))
            .filter(users::deleted_at.is_null())
            .filter(users::username.like(format!("%{}%", query)))
            .order_by(users::created_at)
            .offset(pagination.calc_offset().as_uint().into())
            .limit(pagination.limit.as_uint().into())
            .load::<User>(conn)?;

        get_roles_for_users(users, conn)
    })
}

pub fn find_user_by_email(
    email: &Email,
    conn: &mut DbConnection,
) -> Result<Option<User>, DomainError> {
    use crate::schema::users::dsl as users;

    let mb_user = users::users
        .select((
            users::id,
            users::username,
            users::created_at,
            users::deleted_at,
            users::user_uuid,
            users::email_verified,
        ))
        .filter(users::email.eq(email))
        .filter(users::deleted_at.is_null())
        .first::<User>(conn)
        .optional()?;

    Ok(mb_user)
}

pub fn email_exists(
    email: &Email,
    conn: &mut DbConnection,
) -> Result<bool, DomainError> {
    use crate::schema::users::dsl as users;

    let count: i64 = users::users
        .count()
        .filter(users::email.eq(email))
        .filter(users::deleted_at.is_null())
        .get_result(conn)?;

    Ok(count > 0)
}

pub fn insert_new_user(
    nu: NewUser,
    role: RoleEnum,
    hash_cost: u32,
    user_ids_cache: &InstrumentedRedisCache<String, Vec<UserId>>,
    conn: &mut DbConnection,
) -> Result<(UserId, UserWithRoles), DomainError> {
    use crate::schema::roles::dsl as roles;
    use crate::schema::users::dsl as users;
    use crate::schema::users_roles::dsl as users_roles;

    let nu = {
        let mut nu2 = nu;
        let hash = hash(nu2.password.as_str(), hash_cost)?;
        nu2.password = Password::parse_string(hash).map_err(|err| {
            DomainError::new_field_validation_error(err.to_string())
        })?;
        nu2
    };

    conn.transaction(|conn| {
        let email_taken: i64 = users::users
            .count()
            .filter(users::email.eq(&nu.email))
            .filter(users::deleted_at.is_null())
            .get_result(conn)?;

        if email_taken > 0 {
            return Err(DomainError::new_field_validation_error(format!(
                "Email '{}' is already registered",
                nu.email
            )));
        }

        diesel::insert_into(users::users)
            .values(&nu)
            .execute(conn)?;
        let role_id = roles::roles
            .select(roles::id)
            .filter(roles::role_name.eq(role))
            .first::<RoleId>(conn)?;
        let user = users::users
            .select((
                users::id,
                users::username,
                users::created_at,
                users::deleted_at,
                users::user_uuid,
                users::email_verified,
            ))
            .filter(users::username.eq(nu.username))
            .filter(users::deleted_at.is_null())
            .first::<User>(conn)?;

        diesel::insert_into(users_roles::users_roles)
            .values(NewUserRole {
                user_id: user.id,
                role_id,
            })
            .execute(conn)?;

        let roles = get_roles_for_user(&user.id, conn)?;

        let user_with_roles = (
            user.id,
            UserWithRoles {
                username: user.username,
                created_at: user.created_at,
                user_uuid: user.user_uuid,
                roles,
            },
        );

        // Invalidate the cache since we've added a new user
        if let Err(e) = user_ids_cache.remove(&"user_ids".to_owned()) {
            tracing::error!(error = %e, "Failed to invalidate user IDs cache");
        }

        Ok(user_with_roles)
    })
}

pub fn insert_new_user_with_roles(
    nu: NewUser,
    roles: &[RoleEnum],
    hash_cost: u32,
    user_ids_cache: &InstrumentedRedisCache<String, Vec<UserId>>,
    conn: &mut DbConnection,
) -> Result<(UserId, UserWithRoles), DomainError> {
    use crate::schema::roles::dsl as roles_schema;
    use crate::schema::users::dsl as users;
    use crate::schema::users_roles::dsl as users_roles;

    let nu = {
        let mut nu2 = nu;
        let hash = hash(nu2.password.as_str(), hash_cost)?;
        nu2.password = Password::parse_string(hash).map_err(|err| {
            DomainError::new_field_validation_error(err.to_string())
        })?;
        nu2
    };

    conn.transaction(|conn| {
        let email_taken: i64 = users::users
            .count()
            .filter(users::email.eq(&nu.email))
            .filter(users::deleted_at.is_null())
            .get_result(conn)?;

        if email_taken > 0 {
            return Err(DomainError::new_field_validation_error(format!(
                "Email '{}' is already registered",
                nu.email
            )));
        }

        diesel::insert_into(users::users)
            .values(&nu)
            .execute(conn)?;

        let user = users::users
            .select((
                users::id,
                users::username,
                users::created_at,
                users::deleted_at,
                users::user_uuid,
                users::email_verified,
            ))
            .filter(users::username.eq(nu.username))
            .filter(users::deleted_at.is_null())
            .first::<User>(conn)?;

        for role in roles {
            let role_id = roles_schema::roles
                .select(roles_schema::id)
                .filter(roles_schema::role_name.eq(role))
                .first::<RoleId>(conn)?;

            diesel::insert_into(users_roles::users_roles)
                .values(NewUserRole {
                    user_id: user.id,
                    role_id,
                })
                .execute(conn)?;
        }

        let roles = get_roles_for_user(&user.id, conn)?;

        let user_with_roles = (
            user.id,
            UserWithRoles {
                username: user.username,
                created_at: user.created_at,
                user_uuid: user.user_uuid,
                roles,
            },
        );

        if let Err(e) = user_ids_cache.remove(&"user_ids".to_owned()) {
            tracing::error!(error = %e, "Failed to invalidate user IDs cache");
        }

        Ok(user_with_roles)
    })
}

pub fn insert_new_regular_user(
    nu: NewUser,
    hash_cost: u32,
    user_ids_cache: &InstrumentedRedisCache<String, Vec<UserId>>,
    conn: &mut DbConnection,
) -> Result<(UserId, UserWithRoles), DomainError> {
    insert_new_user(nu, RoleEnum::RoleUser, hash_cost, user_ids_cache, conn)
}

/// Update the authenticated user's profile fields.
pub fn update_user_profile(
    uuid: &UserUuid,
    updates: UpdateUserProfile,
    conn: &mut DbConnection,
) -> Result<(UserId, UserWithRoles), DomainError> {
    use crate::schema::users::dsl as users;

    conn.transaction(|conn| {
        let existing_id = users::users
            .select((users::id, users::deleted_at))
            .filter(users::user_uuid.eq(uuid))
            .first::<(UserId, Option<chrono::NaiveDateTime>)>(conn)
            .optional()?;

        let existing_id = match existing_id {
            None => {
                return Err(DomainError::new_entity_does_not_exist_error(
                    format!("User not found: {}", uuid),
                ))
            }
            Some((_, Some(_))) => {
                return Err(DomainError::new_account_deleted_error(format!(
                    "User {} is deleted",
                    uuid
                )))
            }
            Some((id, None)) => id,
        };

        let new_username = updates.username;

        if let Some(ref username) = new_username {
            let taken = users::users
                .select(users::id)
                .filter(users::username.eq(username))
                .filter(users::deleted_at.is_null())
                .filter(users::id.ne(existing_id))
                .first::<UserId>(conn)
                .optional()?;

            if taken.is_some() {
                return Err(DomainError::new_field_validation_error(format!(
                    "Username '{}' is already taken",
                    username.as_str()
                )));
            }

            match diesel::update(users::users.filter(users::id.eq(existing_id)))
                .set(users::username.eq(username))
                .execute(conn)
            {
                Ok(_) => {}
                Err(diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    _,
                )) => {
                    return Err(DomainError::new_field_validation_error(
                        format!(
                            "Username '{}' is already taken",
                            username.as_str()
                        ),
                    ));
                }
                Err(e) => return Err(e.into()),
            }
        }

        if let Some(ref email) = updates.email {
            let already_taken = users::users
                .select(users::id)
                .filter(users::email.eq(email))
                .filter(users::deleted_at.is_null())
                .filter(users::id.ne(existing_id))
                .first::<UserId>(conn)
                .optional()?;

            if already_taken.is_some() {
                return Err(DomainError::new_field_validation_error(format!(
                    "Email '{}' is already in use",
                    email.as_str()
                )));
            }

            match diesel::update(users::users.filter(users::id.eq(existing_id)))
                .set(users::email.eq(email))
                .execute(conn)
            {
                Ok(_) => {}
                Err(diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    _,
                )) => {
                    return Err(DomainError::new_field_validation_error(
                        format!("Email '{}' is already in use", email.as_str()),
                    ));
                }
                Err(e) => return Err(e.into()),
            }
        }

        let user = users::users
            .select((
                users::id,
                users::username,
                users::created_at,
                users::deleted_at,
                users::user_uuid,
                users::email_verified,
            ))
            .filter(users::id.eq(existing_id))
            .first::<User>(conn)?;

        let roles = get_roles_for_user(&existing_id, conn)?;

        Ok((existing_id, UserWithRoles::from_user(&user, &roles)))
    })
}

/// Soft-delete a user by setting deleted_at timestamp.
/// Returns an error if the user is already deleted or doesn't exist.
pub fn soft_delete_user(
    uuid: &UserUuid,
    conn: &mut DbConnection,
) -> Result<(), DomainError> {
    use crate::schema::users::dsl as users;

    let existing = users::users
        .select((users::id, users::deleted_at))
        .filter(users::user_uuid.eq(uuid))
        .first::<(UserId, Option<chrono::NaiveDateTime>)>(conn)
        .optional()?;

    match existing {
        None => Err(DomainError::new_entity_does_not_exist_error(format!(
            "User not found: {}",
            uuid
        ))),
        Some((_, Some(_))) => Err(DomainError::new_account_deleted_error(
            format!("User {} is already deleted", uuid),
        )),
        Some((id, None)) => {
            conn.transaction::<_, DomainError, _>(|conn| {
                use crate::schema::jobs::dsl as jobs;

                diesel::update(users::users.filter(users::id.eq(id)))
                    .set(users::deleted_at.eq(chrono::Utc::now().naive_utc()))
                    .execute(conn)?;

                diesel::update(jobs::jobs.filter(jobs::started_by.eq(id)))
                    .set(jobs::started_by.eq(None::<i32>))
                    .execute(conn)?;

                Ok(())
            })?;

            tracing::info!(user_uuid = %uuid, "User soft-deleted");
            Ok(())
        }
    }
}

/// Delete a user's avatar from MinIO.
/// Non-fatal: returns Ok even if the avatar doesn't exist.
pub async fn delete_user_avatar(
    uuid: &UserUuid,
    minio: &minior::Minio,
    bucket_name: &str,
) -> Result<(), DomainError> {
    let object_key = format!("avatars/{}", uuid);

    match minio
        .client
        .delete_object()
        .bucket(bucket_name)
        .key(&object_key)
        .send()
        .await
    {
        Ok(_) => {
            tracing::info!(user_uuid = %uuid, object_key = %object_key, "Avatar deleted from MinIO");
            Ok(())
        }
        Err(e) => {
            let err_str = format!("{:?}", e);
            if err_str.contains("404") || err_str.contains("NoSuchKey") {
                tracing::warn!(user_uuid = %uuid, object_key = %object_key, "Avatar not found, skipping deletion");
                Ok(())
            } else {
                Err(DomainError::new_internal_error(format!(
                    "Failed to delete avatar from MinIO: {}",
                    err_str
                )))
            }
        }
    }
}

/// Find a user by email for OAuth login.
pub fn find_oauth_user_by_email(
    email: &str,
    conn: &mut DbConnection,
) -> Result<Option<OAuthUserLookup>, DomainError> {
    use crate::schema::users::dsl as users;

    let mb_user = users::users
        .select((
            users::id,
            users::username,
            users::email,
            users::password,
            users::oauth_provider,
            users::oauth_uid,
            users::user_uuid,
            users::email_verified,
        ))
        .filter(users::email.eq(email))
        .filter(users::deleted_at.is_null())
        .first::<OAuthUserLookup>(conn)
        .optional()?;

    Ok(mb_user)
}

/// Find or create a user from OAuth data.
/// Returns the user with roles, and whether it was a new registration.
pub fn find_or_create_oauth_user(
    email: &str,
    provider: &OAuthProvider,
    provider_uid: &str,
    _display_name: Option<&str>,
    _hash_cost: u32,
    user_ids_cache: &InstrumentedRedisCache<String, Vec<UserId>>,
    conn: &mut DbConnection,
) -> Result<((UserId, UserWithRoles), bool), DomainError> {
    use crate::schema::users::dsl as users;

    conn.transaction(|conn| {
        // Check if user already exists with this OAuth account
        let existing_oauth = users::users
            .select((
                users::id,
                users::username,
                users::created_at,
                users::deleted_at,
                users::user_uuid,
                users::email_verified,
            ))
            .filter(users::oauth_provider.eq(Some(provider.clone())))
            .filter(users::oauth_uid.eq(Some(provider_uid.to_string())))
            .filter(users::deleted_at.is_null())
            .first::<User>(conn)
            .optional()?;

        if let Some(user) = existing_oauth {
            let roles = get_roles_for_user(&user.id, conn)?;
            return Ok((
                (
                    user.id,
                    UserWithRoles {
                        username: user.username,
                        created_at: user.created_at,
                        user_uuid: user.user_uuid,
                        roles,
                    },
                ),
                false,
            ));
        }

        // Check if user exists with this email
        let existing_email = users::users
            .select((
                users::id,
                users::username,
                users::created_at,
                users::deleted_at,
                users::oauth_provider,
                users::oauth_uid,
                users::user_uuid,
            ))
            .filter(users::email.eq(email))
            .filter(users::deleted_at.is_null())
            .first::<(
                UserId,
                Username,
                chrono::NaiveDateTime,
                Option<chrono::NaiveDateTime>,
                Option<OAuthProvider>,
                Option<String>,
                UserUuid,
            )>(conn)
            .optional()?;

        if let Some((
            uid,
            username,
            created_at,
            _,
            existing_provider,
            _existing_uid,
            user_uuid,
        )) = existing_email
        {
            // User exists with email but different or no OAuth provider
            if existing_provider.is_none()
                || existing_provider.as_ref() != Some(provider)
            {
                // Link the OAuth account to existing user
                diesel::update(users::users.filter(users::id.eq(&uid)))
                    .set((
                        users::oauth_provider.eq(Some(provider.clone())),
                        users::oauth_uid.eq(Some(provider_uid.to_string())),
                    ))
                    .execute(conn)?;

                let roles = get_roles_for_user(&uid, conn)?;
                return Ok((
                    (
                        uid,
                        UserWithRoles {
                            username,
                            created_at,
                            user_uuid,
                            roles,
                        },
                    ),
                    false,
                ));
            }
        }

        // Create new user
        let username_base = email.split('@').next().unwrap_or("user");
        let mut username = username_base.to_string();
        let mut attempt = 0u32;

        loop {
            let username_check = users::users
                .select(users::id)
                .filter(users::username.eq(&username))
                .first::<UserId>(conn)
                .optional()?;

            if username_check.is_none() {
                break;
            }

            attempt += 1;
            username = format!("{}_{}", username_base, attempt);
        }

        loop {
            match diesel::insert_into(users::users)
                .values((
                    users::username.eq(&username),
                    users::password.eq(""),
                    users::email.eq(email),
                    users::oauth_provider.eq(Some(provider.clone())),
                    users::oauth_uid.eq(Some(provider_uid.to_string())),
                ))
                .execute(conn)
            {
                Ok(_) => break,
                Err(diesel::result::Error::DatabaseError(
                    diesel::result::DatabaseErrorKind::UniqueViolation,
                    _,
                )) => {
                    attempt += 1;
                    username = format!("{}_{}", username_base, attempt);
                }
                Err(e) => return Err(e.into()),
            }
        }

        let user = users::users
            .select((
                users::id,
                users::username,
                users::created_at,
                users::deleted_at,
                users::user_uuid,
                users::email_verified,
            ))
            .filter(users::username.eq(&username))
            .filter(users::deleted_at.is_null())
            .first::<User>(conn)?;

        // Assign default role
        let role_id = {
            use crate::schema::roles::dsl as roles;
            roles::roles
                .select(roles::id)
                .filter(roles::role_name.eq(RoleEnum::RoleUser))
                .first::<RoleId>(conn)?
        };

        {
            use crate::schema::users_roles::dsl as users_roles;
            diesel::insert_into(users_roles::users_roles)
                .values(NewUserRole {
                    user_id: user.id,
                    role_id,
                })
                .execute(conn)?;
        }

        let roles = get_roles_for_user(&user.id, conn)?;

        // Invalidate the cache since we've added a new user
        if let Err(e) = user_ids_cache.remove(&"user_ids".to_owned()) {
            tracing::error!(error = %e, "Failed to invalidate user IDs cache");
        }

        Ok((
            (
                user.id,
                UserWithRoles {
                    username: user.username,
                    created_at: user.created_at,
                    user_uuid: user.user_uuid,
                    roles,
                },
            ),
            true,
        ))
    })
}

pub fn get_profile(
    uuid: &UserUuid,
    conn: &mut DbConnection,
) -> Result<Option<Profile>, DomainError> {
    use crate::schema::profiles::dsl as profiles;
    use crate::schema::users::dsl as users;

    let profile = profiles::profiles
        .inner_join(users::users.on(users::id.eq(profiles::user_id)))
        .filter(users::user_uuid.eq(uuid))
        .select((
            profiles::id,
            profiles::user_id,
            profiles::bio,
            profiles::display_name,
            profiles::location,
            profiles::website_url,
            profiles::social_github,
            profiles::social_twitter,
            profiles::created_at,
            profiles::updated_at,
            users::user_uuid,
        ))
        .first::<Profile>(conn)
        .optional()?;

    Ok(profile)
}

pub fn create_profile(
    uuid: &UserUuid,
    create: CreateProfile,
    conn: &mut DbConnection,
) -> Result<Profile, DomainError> {
    use crate::schema::profiles::dsl as profiles;
    use crate::schema::users::dsl as users;

    let user_id = users::users
        .select(users::id)
        .filter(users::user_uuid.eq(uuid))
        .first::<UserId>(conn)
        .map_err(|_| {
            DomainError::new_entity_does_not_exist_error(
                "User not found".to_string(),
            )
        })?;

    let bio = create.bio;
    let display_name = create.display_name;
    let location = create.location;
    let website_url = create.website_url;
    let social_github = create.social_github;
    let social_twitter = create.social_twitter;

    diesel::insert_into(profiles::profiles)
        .values((
            profiles::user_id.eq(user_id),
            profiles::bio.eq(bio.clone()),
            profiles::display_name.eq(display_name.clone()),
            profiles::location.eq(location.clone()),
            profiles::website_url.eq(website_url.clone()),
            profiles::social_github.eq(social_github.clone()),
            profiles::social_twitter.eq(social_twitter.clone()),
        ))
        .execute(conn)?;

    let profile = profiles::profiles
        .inner_join(users::users.on(users::id.eq(profiles::user_id)))
        .filter(users::user_uuid.eq(uuid))
        .select((
            profiles::id,
            profiles::user_id,
            profiles::bio,
            profiles::display_name,
            profiles::location,
            profiles::website_url,
            profiles::social_github,
            profiles::social_twitter,
            profiles::created_at,
            profiles::updated_at,
            users::user_uuid,
        ))
        .first::<Profile>(conn)?;

    Ok(profile)
}

pub fn update_profile(
    uuid: &UserUuid,
    updates: UpdateProfile,
    conn: &mut DbConnection,
) -> Result<Profile, DomainError> {
    use crate::schema::profiles::dsl as profiles;
    use crate::schema::users::dsl as users;

    let user_id = users::users
        .select(users::id)
        .filter(users::user_uuid.eq(uuid))
        .first::<UserId>(conn)
        .map_err(|_| {
            DomainError::new_entity_does_not_exist_error(
                "User not found".to_string(),
            )
        })?;

    let profile = match get_profile(uuid, conn)? {
        Some(mut p) => {
            let bio = updates.bio.clone();
            let display_name = updates.display_name.clone();
            let location = updates.location.clone();
            let website_url = updates.website_url.clone();
            let social_github = updates.social_github.clone();
            let social_twitter = updates.social_twitter.clone();

            if updates.should_update("bio") {
                p.bio = bio;
            }
            if updates.should_update("display_name") {
                p.display_name = display_name;
            }
            if updates.should_update("location") {
                p.location = location;
            }
            if updates.should_update("website_url") {
                p.website_url = website_url;
            }
            if updates.should_update("social_github") {
                p.social_github = social_github;
            }
            if updates.should_update("social_twitter") {
                p.social_twitter = social_twitter;
            }

            diesel::update(
                profiles::profiles.filter(profiles::user_id.eq(user_id)),
            )
            .set((
                profiles::bio.eq(p.bio.clone()),
                profiles::display_name.eq(p.display_name.clone()),
                profiles::location.eq(p.location.clone()),
                profiles::website_url.eq(p.website_url.clone()),
                profiles::social_github.eq(p.social_github.clone()),
                profiles::social_twitter.eq(p.social_twitter.clone()),
            ))
            .execute(conn)?;

            p
        }
        None => {
            let bio = updates.bio;
            let display_name = updates.display_name;
            let location = updates.location;
            let website_url = updates.website_url;
            let social_github = updates.social_github;
            let social_twitter = updates.social_twitter;

            diesel::insert_into(profiles::profiles)
                .values((
                    profiles::user_id.eq(user_id),
                    profiles::bio.eq(bio.clone()),
                    profiles::display_name.eq(display_name.clone()),
                    profiles::location.eq(location.clone()),
                    profiles::website_url.eq(website_url.clone()),
                    profiles::social_github.eq(social_github.clone()),
                    profiles::social_twitter.eq(social_twitter.clone()),
                ))
                .execute(conn)?;

            profiles::profiles
                .inner_join(users::users.on(users::id.eq(profiles::user_id)))
                .filter(users::user_uuid.eq(uuid))
                .select((
                    profiles::id,
                    profiles::user_id,
                    profiles::bio,
                    profiles::display_name,
                    profiles::location,
                    profiles::website_url,
                    profiles::social_github,
                    profiles::social_twitter,
                    profiles::created_at,
                    profiles::updated_at,
                    users::user_uuid,
                ))
                .first::<Profile>(conn)?
        }
    };

    Ok(profile)
}

pub fn get_public_profile(
    uuid: &UserUuid,
    conn: &mut DbConnection,
) -> Result<PublicProfile, DomainError> {
    use crate::schema::profiles::dsl as profiles;
    use crate::schema::users::dsl as users;

    let profile = profiles::profiles
        .inner_join(users::users.on(users::id.eq(profiles::user_id)))
        .filter(users::user_uuid.eq(uuid))
        .select((
            profiles::id,
            profiles::user_id,
            profiles::bio,
            profiles::display_name,
            profiles::location,
            profiles::website_url,
            profiles::social_github,
            profiles::social_twitter,
            profiles::created_at,
            profiles::updated_at,
            users::user_uuid,
        ))
        .first::<Profile>(conn)
        .optional()?;

    match profile {
        Some(p) => Ok(PublicProfile::from(&p)),
        None => Err(DomainError::new_entity_does_not_exist_error(format!(
            "Profile not found for user {}",
            uuid
        ))),
    }
}
