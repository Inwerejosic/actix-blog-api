use sqlx::{postgres::PgPoolOptions, PgPool};

pub async fn create_pool() -> Result<PgPool, sqlx::Error> {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgPoolOptions::new()
        .max_connections(20)
        .connect(&database_url)
        .await
}

pub async fn run_migrations(pool: &PgPool) -> Result<(), sqlx::Error> {
    // Create a transaction to run all migrations atomically
    let mut tx = pool.begin().await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS users (
            id SERIAL PRIMARY KEY,
            username TEXT UNIQUE NOT NULL,
            password TEXT NOT NULL,
            email TEXT UNIQUE NOT NULL,
            last_login TIMESTAMPTZ
        )
        "#,
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS blog_posts (
            id SERIAL PRIMARY KEY,
            user_id INTEGER REFERENCES users(id) ON DELETE SET NULL,
            title TEXT,
            content TEXT,
            cover TEXT
        )
        "#,
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS comments (
            id SERIAL PRIMARY KEY,
            blog_id INTEGER REFERENCES blog_posts(id) ON DELETE CASCADE,
            user_id INTEGER REFERENCES users(id) ON DELETE SET NULL,
            content TEXT
        )
        "#,
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tags (
            id SERIAL PRIMARY KEY,
            name TEXT UNIQUE NOT NULL
        )
        "#,
    )
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS blog_tag (
            blog_id INTEGER REFERENCES blog_posts(id) ON DELETE CASCADE,
            tag_id INTEGER REFERENCES tags(id) ON DELETE CASCADE,
            PRIMARY KEY (blog_id, tag_id)
        )
        "#,
    )
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}
