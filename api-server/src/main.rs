use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{routing::get, Json, Router};
use main_error::MainResult;
use std::env::var;
use std::net::Ipv4Addr;
use std::str::FromStr;
use std::sync::Arc;
use steamid_ng::{SteamID, SteamIDError};
use thiserror::Error;
use tokio::net::TcpListener;
use tokio::signal;
use tracing::{debug, error, instrument};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use ugc_scraper::data::GameMode;
use ugc_scraper::{ScrapeError, UgcClient};

#[derive(Clone, Default)]
struct AppState {
    client: Arc<UgcClient>,
}

#[derive(Debug, Error)]
enum ApiError {
    #[error(transparent)]
    SteamId(#[from] SteamIDError),
    #[error(transparent)]
    Scrape(#[from] ScrapeError),
    #[error("malformed request")]
    Malformed(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        error!(error = ?self, "error while handling request");
        match self {
            Self::SteamId(err) => {
                (StatusCode::UNPROCESSABLE_ENTITY, format!("{:#}", err)).into_response()
            }
            Self::Malformed(err) => {
                (StatusCode::UNPROCESSABLE_ENTITY, format!("{:#}", err)).into_response()
            }
            Self::Scrape(ScrapeError::NotFound) => (StatusCode::NOT_FOUND, "").into_response(),
            Self::Scrape(err) => {
                (StatusCode::INTERNAL_SERVER_ERROR, format!("{:#}", err)).into_response()
            }
        }
    }
}

#[tokio::main]
async fn main() -> MainResult {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "ugc_api_server=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let port = var("PORT")?.parse()?;
    // build our application with a route
    let app = Router::new()
        .route("/", get(handler))
        .route("/player/{id}", get(player))
        .route("/player/{id}/history", get(player_history))
        .route("/teams/{format}", get(teams))
        .route("/transactions/{format}", get(transactions))
        .route("/team/{id}", get(team))
        .route("/team/{id}/roster", get(team_roster))
        .route("/team/{id}/matches", get(team_matches))
        .route("/match/{id}", get(match_page))
        .route("/maps/{format}", get(map_history))
        .with_state(AppState::default());

    let listener = TcpListener::bind((Ipv4Addr::new(127, 0, 0, 1), port)).await?;
    tracing::info!("listening on http://{}", listener.local_addr().unwrap());
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();

    Ok(())
}

async fn handler() -> &'static str {
    include_str!("../README.md")
}

#[instrument(skip(state))]
async fn player(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let steam_id = SteamID::try_from(id.as_str())?;
    debug!(player = steam_id.steam3(), "requesting player");
    let response = state.client.player(steam_id).await?;
    Ok(Json(response))
}

#[instrument(skip(state))]
async fn player_history(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let steam_id = SteamID::try_from(id.as_str())?;
    debug!(player = steam_id.steam3(), "requesting player history");
    let response = state.client.player_team_history(steam_id).await?;
    Ok(Json(response))
}

#[instrument(skip(state))]
async fn teams(
    Path(format): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let mode = match GameMode::from_str(&format) {
        Ok(mode) => mode,
        _ => return Err(ApiError::Malformed(format!("invalid game mode {}", format))),
    };
    let response = state.client.teams(mode).await?;
    Ok(Json(response))
}

#[instrument(skip(state))]
async fn transactions(
    Path(format): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let mode = match GameMode::from_str(&format) {
        Ok(mode) => mode,
        _ => return Err(ApiError::Malformed(format!("invalid game mode {}", format))),
    };
    let response = state.client.transactions(mode).await?;
    Ok(Json(response))
}

#[instrument(skip(state))]
async fn team(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    debug!(team = id, "requesting team");
    let response = state.client.team(id).await?;
    Ok(Json(response))
}

#[instrument(skip(state))]
async fn team_roster(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    debug!(team = id, "requesting team roster");
    let response = state.client.team_roster_history(id).await?;
    Ok(Json(response))
}

#[instrument(skip(state))]
async fn team_matches(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    debug!(team = id, "requesting team matches");
    let response = state.client.team_matches(id).await?;
    Ok(Json(response))
}

#[instrument(skip(state))]
async fn match_page(
    Path(id): Path<u32>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    debug!(r#match = id, "requesting match");
    let response = state.client.match_info(id).await?;
    Ok(Json(response))
}

#[instrument(skip(state))]
async fn map_history(
    Path(format): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let mode = match GameMode::from_str(&format) {
        Ok(mode) => mode,
        _ => return Err(ApiError::Malformed(format!("invalid game mode {}", format))),
    };
    let response = state.client.map_history(mode).await?;
    Ok(Json(response))
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}
