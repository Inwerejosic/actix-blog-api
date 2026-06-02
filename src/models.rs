use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

// ---------- Database Models ----------
#[derive(Debug, Serialize, FromRow, Clone)]
pub struct User {
    pub id: i32,
    pub username: Option<String>,
    pub password: Option<String>,
    pub email: Option<String>,
    pub last_login: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct BlogPost {
    pub id: i32,
    pub user_id: Option<i32>,
    pub title: Option<String>,
    pub content: Option<String>,
    pub cover: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct Tag {
    pub id: i32,
    pub name: Option<String>,
}

// ---------- Response DTOs (without password) ----------
#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: i32,
    pub username: Option<String>,
    pub email: Option<String>,
    pub last_login: Option<NaiveDateTime>,
}

impl From<User> for UserResponse {
    fn from(u: User) -> Self {
        UserResponse {
            id: u.id,
            username: u.username,
            email: u.email,
            last_login: u.last_login,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PostWithTags {
    #[serde(flatten)]
    pub post: BlogPost,
    pub tags: Vec<Tag>,
}

// ---------- Input DTOs ----------
#[derive(Debug, Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub password: String,
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUser {
    pub username: Option<String>,
    pub password: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreatePost {
    pub user_id: i32,
    pub title: String,
    pub content: String,
    pub cover: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTag {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginCredentials {
    pub username: String,
    pub password: String,
}