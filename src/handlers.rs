use actix_session::Session;
use actix_web::{web, HttpResponse};
use sqlx::PgPool;
use chrono::Utc;

use crate::auth::{self, hash_password};
use crate::error::ApiError;
use crate::models::*;

type DbPool = web::Data<PgPool>;

// ---------- Auth endpoints (thin wrappers around auth module) ----------
pub async fn login_handler(
    session: Session,
    pool: DbPool,
    creds: web::Json<LoginCredentials>,
) -> Result<HttpResponse, ApiError> {
    let user = auth::login(session, pool.get_ref(), creds.into_inner()).await?;
    Ok(HttpResponse::Ok().json(user))
}

pub async fn logout_handler(session: Session) -> Result<HttpResponse, ApiError> {
    auth::logout(session).await
}

pub async fn me_handler(session: Session, pool: DbPool) -> Result<HttpResponse, ApiError> {
    let user = auth::current_user(session, pool.get_ref()).await?;
    Ok(HttpResponse::Ok().json(user))
}

// ---------- Users (CRUD with password hashing) ----------
pub async fn list_users(pool: DbPool) -> Result<HttpResponse, ApiError> {
    let users: Vec<UserResponse> = sqlx::query_as::<_, User>(
        "SELECT id, username, password, email, last_login FROM users"
    )
    .fetch_all(pool.get_ref())
    .await?
    .into_iter()
    .map(UserResponse::from)
    .collect();
    Ok(HttpResponse::Ok().json(users))
}

pub async fn get_user(pool: DbPool, path: web::Path<i32>) -> Result<HttpResponse, ApiError> {
    let id = path.into_inner();
    let user: Option<User> = sqlx::query_as::<_, User>(
        "SELECT id, username, password, email, last_login FROM users WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(pool.get_ref())
    .await?;
    match user {
        Some(u) => Ok(HttpResponse::Ok().json(UserResponse::from(u))),
        None => Err(ApiError::NotFound),
    }
}

pub async fn create_user(pool: DbPool, payload: web::Json<CreateUser>) -> Result<HttpResponse, ApiError> {
    let hashed = hash_password(&payload.password)?;
    let now = Utc::now().naive_utc();
    let user: User = sqlx::query_as::<_, User>(
        "INSERT INTO users (username, password, email, last_login) VALUES ($1, $2, $3, $4) RETURNING id, username, password, email, last_login"
    )
    .bind(&payload.username)
    .bind(hashed)
    .bind(&payload.email)
    .bind(now)
    .fetch_one(pool.get_ref())
    .await?;
    Ok(HttpResponse::Created().json(UserResponse::from(user)))
}

pub async fn update_user(
    pool: DbPool,
    path: web::Path<i32>,
    payload: web::Json<UpdateUser>,
) -> Result<HttpResponse, ApiError> {
    let id = path.into_inner();
    let current: Option<User> = sqlx::query_as::<_, User>(
        "SELECT id, username, password, email, last_login FROM users WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(pool.get_ref())
    .await?;
    let current = current.ok_or(ApiError::NotFound)?;

    let new_username = payload.username.clone().unwrap_or_else(|| current.username.clone().unwrap_or_default());
    let new_password = if let Some(pw) = &payload.password {
        hash_password(pw)?
    } else {
        current.password.clone().unwrap_or_default()
    };
    let new_email = payload.email.clone().unwrap_or_else(|| current.email.clone().unwrap_or_default());

    let updated: User = sqlx::query_as::<_, User>(
        "UPDATE users SET username = $1, password = $2, email = $3 WHERE id = $4 RETURNING id, username, password, email, last_login"
    )
    .bind(new_username)
    .bind(new_password)
    .bind(new_email)
    .bind(id)
    .fetch_one(pool.get_ref())
    .await?;
    Ok(HttpResponse::Ok().json(UserResponse::from(updated)))
}

pub async fn delete_user(pool: DbPool, path: web::Path<i32>) -> Result<HttpResponse, ApiError> {
    let id = path.into_inner();
    let rows = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(id)
        .execute(pool.get_ref())
        .await?
        .rows_affected();
    if rows == 0 {
        Err(ApiError::NotFound)
    } else {
        Ok(HttpResponse::NoContent().finish())
    }
}

// ---------- Blog Posts (unchanged except syntax) ----------
pub async fn list_posts(pool: DbPool) -> Result<HttpResponse, ApiError> {
    let posts: Vec<BlogPost> = sqlx::query_as::<_, BlogPost>(
        "SELECT id, user_id, title, content, cover FROM blog_posts ORDER BY id DESC"
    )
    .fetch_all(pool.get_ref())
    .await?;
    Ok(HttpResponse::Ok().json(posts))
}

pub async fn create_post(pool: DbPool, payload: web::Json<CreatePost>) -> Result<HttpResponse, ApiError> {
    let post: BlogPost = sqlx::query_as::<_, BlogPost>(
        "INSERT INTO blog_posts (user_id, title, content, cover) VALUES ($1, $2, $3, $4) RETURNING id, user_id, title, content, cover"
    )
    .bind(payload.user_id)
    .bind(&payload.title)
    .bind(&payload.content)
    .bind(&payload.cover)
    .fetch_one(pool.get_ref())
    .await?;
    Ok(HttpResponse::Created().json(post))
}

pub async fn get_post_with_tags(pool: DbPool, path: web::Path<i32>) -> Result<HttpResponse, ApiError> {
    let post_id = path.into_inner();
    let post: Option<BlogPost> = sqlx::query_as::<_, BlogPost>(
        "SELECT id, user_id, title, content, cover FROM blog_posts WHERE id = $1"
    )
    .bind(post_id)
    .fetch_optional(pool.get_ref())
    .await?;
    let post = post.ok_or(ApiError::NotFound)?;

    let tags: Vec<Tag> = sqlx::query_as::<_, Tag>(
        r#"
        SELECT t.id, t.name FROM tags t
        INNER JOIN blog_tag bt ON bt.tag_id = t.id
        WHERE bt.blog_id = $1
        "#
    )
    .bind(post_id)
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(PostWithTags { post, tags }))
}

// ---------- Tags ----------
pub async fn list_tags(pool: DbPool) -> Result<HttpResponse, ApiError> {
    let tags: Vec<Tag> = sqlx::query_as::<_, Tag>("SELECT id, name FROM tags")
        .fetch_all(pool.get_ref())
        .await?;
    Ok(HttpResponse::Ok().json(tags))
}

pub async fn create_tag(pool: DbPool, payload: web::Json<CreateTag>) -> Result<HttpResponse, ApiError> {
    let tag: Tag = sqlx::query_as::<_, Tag>(
        "INSERT INTO tags (name) VALUES ($1) ON CONFLICT (name) DO NOTHING RETURNING id, name"
    )
    .bind(&payload.name)
    .fetch_one(pool.get_ref())
    .await?;
    Ok(HttpResponse::Created().json(tag))
}

pub async fn add_tag_to_post(pool: DbPool, path: web::Path<(i32, i32)>) -> Result<HttpResponse, ApiError> {
    let (post_id, tag_id) = path.into_inner();

    let post_exists: Option<BlogPost> = sqlx::query_as::<_, BlogPost>("SELECT id FROM blog_posts WHERE id = $1")
        .bind(post_id)
        .fetch_optional(pool.get_ref())
        .await?;
    if post_exists.is_none() {
        return Err(ApiError::NotFound);
    }

    let tag_exists: Option<Tag> = sqlx::query_as::<_, Tag>("SELECT id FROM tags WHERE id = $1")
        .bind(tag_id)
        .fetch_optional(pool.get_ref())
        .await?;
    if tag_exists.is_none() {
        return Err(ApiError::NotFound);
    }

    sqlx::query("INSERT INTO blog_tag (blog_id, tag_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
        .bind(post_id)
        .bind(tag_id)
        .execute(pool.get_ref())
        .await?;

    Ok(HttpResponse::Created().finish())
}

pub async fn remove_tag_from_post(pool: DbPool, path: web::Path<(i32, i32)>) -> Result<HttpResponse, ApiError> {
    let (post_id, tag_id) = path.into_inner();
    let rows = sqlx::query("DELETE FROM blog_tag WHERE blog_id = $1 AND tag_id = $2")
        .bind(post_id)
        .bind(tag_id)
        .execute(pool.get_ref())
        .await?
        .rows_affected();

    if rows == 0 {
        Err(ApiError::NotFound)
    } else {
        Ok(HttpResponse::NoContent().finish())
    }
}