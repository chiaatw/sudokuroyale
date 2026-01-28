use chrono::{Duration, Utc};
use uuid::Uuid;

use crate::user::{
    auth::{hash_password, verify_password},
    error::{AuthError, AuthResult},
    model::User,
    repository::UserRepository,
    reset_token::ResetToken,
    reset_token_repository::ResetTokenRepository,
    session::Session,
    session_repository::SessionRepository,
    validation::{validate_email, validate_password, validate_username},
};

pub async fn register_user(
    repo: &UserRepository,
    username: &str,
    email: &str,
    password: &str,
) -> AuthResult<()> {
    validate_username(username).map_err(|_| AuthError::InvalidUsername)?;
    validate_email(email).map_err(|_| AuthError::InvalidEmail)?;
    validate_password(password).map_err(|_| AuthError::InvalidPassword)?;

    if repo.find_by_username(username).await?.is_some() {
        return Err(AuthError::UsernameExists);
    }

    if repo.find_by_email(email).await?.is_some() {
        return Err(AuthError::EmailExists);
    }

    let password_hash = hash_password(password).map_err(|_| AuthError::PasswordHashingFailed)?;

    let user = User {
        id: Uuid::new_v4(),
        username: username.to_string(),
        email: email.to_string(),
        password_hash,
        created_at: chrono::Utc::now(),

    };

    repo.add_user(&user)
        .await
        .map_err(|_| AuthError::DatabaseError)?;

    Ok(())
}

pub async fn login_user(
    repo: &UserRepository,
    session_repo: &mut SessionRepository,
    username: &str,
    password: &str,
) -> AuthResult<Uuid> {
    let user = repo
        .find_by_username(username)
        .await?
        .ok_or(AuthError::UserNotFound)?;

    let ok = verify_password(password, &user.password_hash)
        .map_err(|_| AuthError::InvalidPasswordLogin)?;

    if !ok {
        return Err(AuthError::InvalidPasswordLogin);
    }

    let session = Session {
        id: Uuid::new_v4(),
        user_id: user.id,
        created_at: Utc::now(),
        expires_at: Utc::now() + Duration::hours(24),
    };

    let session_id = session.id;
    session_repo.add_session(session);

    Ok(session_id)
}

pub fn is_logged_in(session_repo: &SessionRepository, session_id: &Uuid) -> bool {
    session_repo.validate_session(session_id)
}

pub fn logout_user(session_repo: &mut SessionRepository, session_id: &Uuid) {
    session_repo.remove_session(session_id);
}

pub async fn get_user_from_session<'a>(
    session_repo: &'a SessionRepository,
    user_repo: &'a UserRepository,
    session_id: &Uuid,
) -> Option<User> {
    // 1. session finden
    let session = session_repo.find_by_session_id(session_id)?;

    //2.  checken ob session gültig ist
    if !session.is_valid() {
        return None;
    }

    //3. user anhand der user_id aus session holen
    user_repo.find_by_id(&session.user_id).await.ok().flatten()
}

pub async fn change_password(
    session_repo: &SessionRepository,
    user_repo: &UserRepository,
    session_id: &Uuid,
    old_password: &str,
    new_password: &str,
) -> AuthResult<()> {
    // session check
    let session = session_repo
        .find_by_session_id(session_id)
        .ok_or(AuthError::SessionExpired)?;

    if !session.is_valid() {
        return Err(AuthError::SessionExpired);
    }

    let user_id = session.user_id;

    //user holen
    let user = user_repo
        .find_by_id(&user_id)
        .await?
        .ok_or(AuthError::UserNotFound)?;

    // altes passwort checken
    let ok = verify_password(old_password, &user.password_hash)
        .map_err(|_| AuthError::InvalidPasswordLogin)?;

    if !ok {
        return Err(AuthError::InvalidPasswordLogin);
    }

    //neues passwort validieren
    validate_password(new_password).map_err(|_| AuthError::InvalidPassword)?;

    //neues passwort hashen
    let new_hash = hash_password(new_password).map_err(|_| AuthError::PasswordHashingFailed)?;

    //passwort im user repository updaten
    user_repo
        .update_password(&user_id, new_hash)
        .await
        .map_err(|_| AuthError::DatabaseError)?;

    Ok(())
}

pub async fn request_password_reset(
    repo: &UserRepository,
    token_repo: &mut ResetTokenRepository,
    email: &str,
) -> AuthResult<Uuid> {
    let user = repo
        .find_by_email(email)
        .await?
        .ok_or(AuthError::UserNotFound)?;

    let token = ResetToken::new(user.id);
    let token_id = token.token;

    token_repo.add_token(token);

    Ok(token_id)
}

pub async fn reset_password(
    repo: &UserRepository,
    token_repo: &mut ResetTokenRepository,
    token: &Uuid,
    new_password: &str,
) -> AuthResult<()> {
    let token_data = token_repo
        .find_token(token)
        .ok_or(AuthError::TokenInvalid)?;

    if !token_data.is_valid() {
        return Err(AuthError::TokenExpired);
    }

    validate_password(new_password).map_err(|_| AuthError::InvalidPassword)?;

    let new_hash = hash_password(new_password).map_err(|_| AuthError::PasswordHashingFailed)?;

    repo.update_password(&token_data.user_id, new_hash).await?;

    token_repo.remove_token(token);

    Ok(())
}
