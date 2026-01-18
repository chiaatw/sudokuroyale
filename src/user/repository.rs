use sqlx::PgPool;
use uuid::Uuid;
use crate::user::model::User;
use chrono::{NaiveDateTime, DateTime, Utc};

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
        sqlx::query!(
            r#"
            INSERT INTO users (id, username, email, password_hash, created_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            user.id,
            user.username,
            user.email,
            user.password_hash,
            user.created_at.naive_utc(), 
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // ---------------------------
    // READ
    // ---------------------------
    pub async fn find_by_id(&self, id: &Uuid) -> Result<Option<User>, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            SELECT id, username, email, password_hash, created_at
            FROM users
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| User {
            id: r.id,
            username: r.username,
            email: r.email,
            password_hash: r.password_hash,
            created_at: DateTime::<Utc>::from_utc(r.created_at, Utc),
        }))
    }

    pub async fn find_by_username(&self, username: &str) -> Result<Option<User>, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            SELECT id, username, email, password_hash, created_at
            FROM users
            WHERE username = $1
            "#,
            username
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| User {
            id: r.id,
            username: r.username,
            email: r.email,
            password_hash: r.password_hash,
            created_at: DateTime::<Utc>::from_utc(r.created_at, Utc),
        }))
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error> {
        let row = sqlx::query!(
            r#"
            SELECT id, username, email, password_hash, created_at
            FROM users
            WHERE email = $1
            "#,
            email
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| User {
            id: r.id,
            username: r.username,
            email: r.email,
            password_hash: r.password_hash,
            created_at: DateTime::<Utc>::from_utc(r.created_at, Utc),
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
        sqlx::query!(
            r#"
            UPDATE users
            SET password_hash = $1
            WHERE id = $2
            "#,
            new_hash,
            user_id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}


 