mod auth;
mod db;
mod error;
mod handlers;
mod models;

use actix_session::{SessionMiddleware, storage::RedisSessionStore};
use actix_web::{web, App, HttpServer, cookie::Key};
use dotenv::dotenv;
use std::time::Duration;
use tokio::time::sleep;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    // ----- Redis session store -----
    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");
    let store = RedisSessionStore::new(redis_url).await.unwrap();

    // ----- Secret key (from environment) -----
    let secret_key = {
        let key_hex = std::env::var("SECRET_KEY").expect("SECRET_KEY must be set");
        let mut key_bytes = [0u8; 64];
        hex::decode_to_slice(key_hex, &mut key_bytes).expect("SECRET_KEY must be 128 hex characters");
        Key::from(&key_bytes)
    };

    // ----- Database pool -----
    let pool = db::create_pool().await.expect("Failed to create DB pool");

    // Wait for DB readiness
    let mut db_ready = false;
    for attempt in 1..=10 {
        match pool.acquire().await {
            Ok(_) => {
                println!("Database ready after {} attempts", attempt);
                db_ready = true;
                break;
            }
            Err(e) => {
                eprintln!("Database not ready (attempt {}): {}", attempt, e);
                sleep(Duration::from_secs(2)).await;
            }
        }
    }
    if !db_ready {
        panic!("Database not reachable after 10 attempts");
    }

    db::run_migrations(&pool).await.expect("Migrations failed");

    // ----- Start server -----
    HttpServer::new(move || {
        App::new()
            .wrap(SessionMiddleware::new(
                store.clone(),
                secret_key.clone(),
            ))
            .app_data(web::Data::new(pool.clone()))
            // Auth endpoints
            .service(web::resource("/login").route(web::post().to(handlers::login_handler)))
            .service(web::resource("/logout").route(web::post().to(handlers::logout_handler)))
            .service(web::resource("/me").route(web::get().to(handlers::me_handler)))
            // User CRUD
            .service(web::resource("/users").route(web::get().to(handlers::list_users)).route(web::post().to(handlers::create_user)))
            .service(web::resource("/users/{id}").route(web::get().to(handlers::get_user)).route(web::put().to(handlers::update_user)).route(web::delete().to(handlers::delete_user)))
            // Blog posts
            .service(web::resource("/posts").route(web::get().to(handlers::list_posts)).route(web::post().to(handlers::create_post)))
            .service(web::resource("/posts/{id}").route(web::get().to(handlers::get_post_with_tags)))
            // Tags
            .service(web::resource("/tags").route(web::get().to(handlers::list_tags)).route(web::post().to(handlers::create_tag)))
            .service(web::resource("/posts/{post_id}/tags/{tag_id}").route(web::post().to(handlers::add_tag_to_post)).route(web::delete().to(handlers::remove_tag_from_post)))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}