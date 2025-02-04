use std::collections::HashMap;
use std::net;
use std::sync::{Arc, Mutex};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};

use clap::Parser;

use moq_api::{ApiError, Origin};

/// Runs a HTTP API to create/get origins for broadcasts.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct ServerConfig {
    /// Listen for HTTP requests on the given address
    #[arg(long, default_value = "[::]:80")]
    pub bind: net::SocketAddr,
}

pub struct Server {
    config: ServerConfig,
}

impl Server {
    pub fn new(config: ServerConfig) -> Self {
        Self { config }
    }

    pub async fn run(self) -> Result<(), ApiError> {
        let state = Arc::new(Mutex::new(HashMap::new()));

        let app = Router::new()
            .route(
                "/origin/*namespace",
                get(get_origin)
                    .post(set_origin)
                    .delete(delete_origin)
                    .patch(patch_origin),
            )
            .with_state(state);

        log::info!("serving requests: bind={}", self.config.bind);

        let listener = tokio::net::TcpListener::bind(&self.config.bind).await?;
        axum::serve(listener, app.into_make_service()).await?;

        Ok(())
    }
}

async fn get_origin(
    Path(namespace): Path<String>,
    State(state): State<Arc<Mutex<HashMap<String, Origin>>>>,
) -> Result<Json<Origin>, AppError> {
    let state = state.lock().unwrap();
    let origin = state.get(&namespace).ok_or(AppError::NotFound)?;
    Ok(Json(origin.clone()))
}

async fn set_origin(
    State(state): State<Arc<Mutex<HashMap<String, Origin>>>>,
    Path(namespace): Path<String>,
    Json(origin): Json<Origin>,
) -> Result<(), AppError> {
    let mut state = state.lock().unwrap();

    if state.contains_key(&namespace) {
        return Err(AppError::Duplicate);
    }

    state.insert(namespace, origin);
    Ok(())
}

async fn delete_origin(
    Path(namespace): Path<String>,
    State(state): State<Arc<Mutex<HashMap<String, Origin>>>>,
) -> Result<(), AppError> {
    let mut state = state.lock().unwrap();
    match state.remove(&namespace) {
        Some(_) => Ok(()),
        None => Err(AppError::NotFound),
    }
}

// Update the expiration deadline.
async fn patch_origin(
    Path(namespace): Path<String>,
    State(state): State<Arc<Mutex<HashMap<String, Origin>>>>,
    Json(origin): Json<Origin>,
) -> Result<(), AppError> {
    let mut state = state.lock().unwrap();

    if let Some(existing_origin) = state.get(&namespace) {
        if existing_origin != &origin {
            return Err(AppError::Duplicate);
        }
    } else {
        return Err(AppError::NotFound);
    }

    state.insert(namespace, origin);
    Ok(())
}

#[derive(thiserror::Error, Debug)]
enum AppError {
    #[error("not found")]
    NotFound,

    #[error("duplicate ID")]
    Duplicate,
}

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::NotFound => StatusCode::NOT_FOUND.into_response(),
            AppError::Duplicate => StatusCode::CONFLICT.into_response(),
        }
    }
}
