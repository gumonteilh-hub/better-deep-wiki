#![cfg(feature = "api")]

use std::sync::Arc;

use axum::{
    body::Body, extract::State, http::StatusCode, response::{IntoResponse}, routing::{get, post}, Json, Router
};
use futures_util::stream::StreamExt;
use qdrant_client::{Qdrant, config::QdrantConfig, qdrant::ListCollectionsResponse};
use serde::{Deserialize, Serialize};
use tracing::error;

/// ═════════════════════ modèles JSON ═════════════════════

#[derive(Deserialize)]
struct ScanRequest {
    repo_path: String,
}

#[derive(Serialize)]
struct ScanResponse {
    repo_identifier: String,
}

#[derive(Deserialize)]
struct AskRequest {
    question: String,
    instructions: String,
    repo_identifier: String,
}

#[derive(Serialize)]
struct RepoListResponse {
    repos: Vec<String>,
}

/// ═════════════════════ global state ═════════════════════
#[derive(Clone)]
struct AppState {
    qdrant: Arc<Qdrant>,
}

/// ═════════════════════ handlers ═════════════════════

/// POST /scan_repo
async fn scan_repo_handler(
    Json(req): Json<ScanRequest>,
) -> Result<Json<ScanResponse>, (StatusCode, String)> {
    let repo_identifier = crate::scan_repo(req.repo_path.clone()).await;

    Ok(Json(ScanResponse { repo_identifier }))
}

/// POST /ask_repo
async fn ask_repo_handler(Json(req): Json<AskRequest>) -> impl IntoResponse {
    let (tx, rx) = tokio::sync::mpsc::channel::<String>(16);

    tokio::spawn(async move {
        let _ = crate::ask_repo(req.question, req.instructions, req.repo_identifier, tx).await;
    });

    let stream = tokio_stream::wrappers::ReceiverStream::new(rx)
        .map(|chunk| Ok::<_, std::io::Error>(chunk.into_bytes()));

    (
        [("Content-Type", "text/plain; charset=utf-8")],
        Body::from_stream(stream),
    )
}

/// GET /repos
async fn list_repos_handler(
    State(state): State<AppState>,
) -> Result<Json<RepoListResponse>, (StatusCode, String)> {
    let resp: ListCollectionsResponse = state.qdrant.list_collections().await.map_err(|e| {
        error!(?e, "Error listing Qdrant collections");
        return (StatusCode::BAD_GATEWAY, e.to_string());
    })?;

    let repos = resp
        .collections
        .into_iter()
        .map(|c| c.name)
        .collect::<Vec<_>>();

    Ok(Json(RepoListResponse { repos }))
}

/// GET /indexable-repos
async fn list_indexable_repos() -> Result<Json<RepoListResponse>, (StatusCode, String)> {
    match crate::collect_repos() {
        Ok(repos) => Ok(Json(RepoListResponse { repos })),
        Err(err) => {
            eprintln!("[indexable-repos] error: {err}");
            Err((StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
        }
    }
}

/// ═════════════════════ public router ═════════════════════

pub fn build_router() -> Router {
    let config = QdrantConfig::from_url("http://localhost:6334").skip_compatibility_check();
    let client = Qdrant::new(config)
        .map_err(|e| e.to_string())
        .expect("Fail to open Qdrant connection");

    let state = AppState {
        qdrant: Arc::new(client),
    };

    Router::new()
        .route("/scan_repo", post(scan_repo_handler))
        .route("/ask_repo", post(ask_repo_handler))
        .route("/repos", get(list_repos_handler))
        .route("/indexable-repos", get(list_indexable_repos))
        .with_state(state)
}
