use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{response::Html, routing::get, Json, Router};
use main_error::MainResult;
use std::env::var;
use std::net::SocketAddr;
use std::sync::Arc;
use steamid_ng::{SteamID, SteamIDError};
use thiserror::Error;
use tracing::{debug, error, instrument};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
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
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        error!(error = ?self, "error while handling request");
        match self {
            Self::SteamId(err) => {
                (StatusCode::UNPROCESSABLE_ENTITY, format!("{:#}", err)).into_response()
            }
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
        .route("/player/:id", get(player))
        .route("/player/:id/history", get(player_history))
        .route("/team/:id", get(team))
        .route("/team/:id/roster", get(team_roster))
        .route("/team/:id/matches", get(team_matches))
        .with_state(AppState::default());

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    tracing::info!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await?;
    Ok(())
}

async fn handler() -> Html<&'static str> {
    Html("<h1>Hello, World!</h1>")
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
