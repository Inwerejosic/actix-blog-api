use actix_session::Session;
use actix_web::HttpResponse;               // removed `web` since unused
use bcrypt::{hash, verify, DEFAULT_COST};
use sqlx::PgPool;

use crate::error::ApiError;
use crate::models::{LoginCredentials, User, UserResponse};

pub fn hash_password(plain: &str) -> Result<String, ApiError> {
    hash(plain, DEFAULT_COST).map_err(|_| ApiError::BadRequest("Password hashing failed".into()))
}

pub fn verify_password(plain: &str, hashed: &str) -> Result<bool, ApiError> {
    verify(plain, hashed).map_err(|_| ApiError::Unauthorized)
}

pub async fn login(
    session: Session,
    pool: &PgPool,
    creds: LoginCredentials,
) -> Result<UserResponse, ApiError> {
    let user: Option<User> = sqlx::query_as::<_, User>(
        "SELECT id, username, password, email, last_login FROM users WHERE username = $1"
    )
    .bind(&creds.username)
    .fetch_optional(pool)
    .await?;

    let user = user.ok_or(ApiError::Unauthorized)?;
    let stored_hash = user.password.as_deref().ok_or(ApiError::Unauthorized)?;

    if !verify_password(&creds.password, stored_hash)? {
        return Err(ApiError::Unauthorized);
    }

    let now = chrono::Utc::now().naive_utc();
    sqlx::query("UPDATE users SET last_login = $1 WHERE id = $2")
        .bind(now)
        .bind(user.id)
        .execute(pool)
        .await?;

    // Now `?` works because ApiError implements From<SessionInsertError>
    session.insert("user_id", user.id)?;
    session.renew();

    Ok(UserResponse::from(user))
}

pub async fn logout(session: Session) -> Result<HttpResponse, ApiError> {
    session.purge();
    Ok(HttpResponse::Ok().json(serde_json::json!({ "message": "Logged out" })))
}

pub async fn current_user(session: Session, pool: &PgPool) -> Result<UserResponse, ApiError> {
    let user_id: i32 = session.get("user_id")?   // now works
        .ok_or(ApiError::Unauthorized)?;

    let user: Option<User> = sqlx::query_as::<_, User>(
        "SELECT id, username, password, email, last_login FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    user.map(UserResponse::from).ok_or(ApiError::Unauthorized)
}

pub fn get_session_user_id(session: &Session) -> Result<i32, ApiError> {
    session.get("user_id")?.ok_or(ApiError::Unauthorized)
}