use crate::user::model::User;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

pub struct UserRepository {
    pool: PgPool,
}

impl UserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    // ---------------------------
    // CREATE
    // ---------------------------
    pub async fn add_user(&self, user: &User) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            INSERT INTO users (id, username, email, password_hash, created_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(user.id)
        .bind(&user.username)
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(user.created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // ---------------------------
    // READ
    // ---------------------------
    pub async fn find_by_id(&self, id: &Uuid) -> Result<Option<User>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT id, username, email, password_hash, created_at
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| User {
            id: r.get("id"),
            username: r.get("username"),
            email: r.get("email"),
            password_hash: r.get("password_hash"),
            created_at: r.get::<DateTime<Utc>, _>("created_at"),
        }))
    }

    pub async fn find_by_username(&self, username: &str) -> Result<Option<User>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT id, username, email, password_hash, created_at
            FROM users
            WHERE username = $1
            "#,
        )
        .bind(username)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| User {
            id: r.get("id"),
            username: r.get("username"),
            email: r.get("email"),
            password_hash: r.get("password_hash"),
            created_at: r.get::<DateTime<Utc>, _>("created_at"),
        }))
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error> {
        let row = sqlx::query(
            r#"
            SELECT id, username, email, password_hash, created_at
            FROM users
            WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| User {
            id: r.get("id"),
            username: r.get("username"),
            email: r.get("email"),
            password_hash: r.get("password_hash"),
            created_at: r.get::<DateTime<Utc>, _>("created_at"),
        }))
    }

    // ---------------------------
    // UPDATE
    // ---------------------------
    pub async fn update_password(
        &self,
        user_id: &Uuid,
        new_hash: String,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            r#"
            UPDATE users
            SET password_hash = $1
            WHERE id = $2
            "#,
        )
        .bind(new_hash)
        .bind(user_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
