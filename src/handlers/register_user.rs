use crate::db::{user, PrismaClient};
use crate::state::AppState;
use axum::{
    extract::{Json, State},
    http::StatusCode,
};
use common::register::{RegisterUserRequest, RegisterUserResponse};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use tracing::error;

#[derive(Debug)]
pub enum RegistrationError {
    Unauthorized,
    UsernameTaken,
    DatabaseError(String),
}

// Convert RegistrationError to StatusCode
impl From<RegistrationError> for StatusCode {
    fn from(error: RegistrationError) -> StatusCode {
        match error {
            RegistrationError::Unauthorized => StatusCode::UNAUTHORIZED,
            RegistrationError::UsernameTaken => StatusCode::BAD_REQUEST,
            RegistrationError::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl std::fmt::Display for RegistrationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegistrationError::Unauthorized => write!(f, "Unauthorized access"),
            RegistrationError::UsernameTaken => write!(f, "Username is already taken"),
            RegistrationError::DatabaseError(err) => write!(f, "Database error: {}", err),
        }
    }
}

fn generate_key(username: &str) -> String {
    let random_part: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(36)
        .map(char::from)
        .collect();
    format!("{}_{}", username, random_part)
}

async fn check_username_exists(
    db: &PrismaClient,
    username: &str,
) -> Result<bool, RegistrationError> {
    db.user()
        .find_first(vec![user::username::equals(username.to_string())])
        .exec()
        .await
        .map(|result| result.is_some())
        .map_err(|e| RegistrationError::DatabaseError(e.to_string()))
}
async fn create_user(
    db: &PrismaClient,
    username: &str,
    access_key: &str,
) -> Result<user::Data, RegistrationError> {
    db.user()
        .create(username.to_string(), access_key.to_string(), vec![])
        .exec()
        .await
        .map_err(|e| RegistrationError::DatabaseError(e.to_string()))
}

pub async fn register_user_handler(
    State(state): State<AppState>,
    Json(payload): Json<RegisterUserRequest>,
) -> Result<Json<RegisterUserResponse>, StatusCode> {
    // Check admin key
    if payload.admin_key != state.admin_key {
        return Err(RegistrationError::Unauthorized.into());
    }

    // Check if username exists
    match check_username_exists(&state.db, &payload.username).await {
        Ok(true) => return Err(RegistrationError::UsernameTaken.into()),
        Ok(false) => {} // Username is available
        Err(error) => {
            error!(
                error = %error,
                username = %payload.username,
                "Database error while checking username existence"
            );
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }

    // Generate a key
    let key = generate_key(&payload.username);

    // Create user
    match create_user(&state.db, &payload.username.clone(), &key).await {
        Ok(_user) => Ok(Json(RegisterUserResponse {
            username: payload.username,
            key,
        })),
        Err(e) => {
            error!(
                error = %e,
                username = %payload.username,
                "Failed to create user in database"
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
