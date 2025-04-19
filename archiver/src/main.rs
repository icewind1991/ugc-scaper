mod archive;
mod client;
mod config;

use crate::archive::Archive;
use crate::client::{UgcClient, UgcClientError};
use crate::config::Config;
use clap::{Parser, Subcommand};
use main_error::MainResult;
use std::path::PathBuf;
use std::pin::pin;
use std::str::FromStr;
use std::time::Duration;
use tokio::time::sleep;
use tokio_stream::StreamExt;
use tracing::{error, info, span, warn, Level};
use ugc_scraper_types::GameMode;

#[derive(Debug, Parser)]
struct Args {
    #[clap(long, short)]
    config: PathBuf,
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Matches,
    Players,
    Teams,
    FixupTeams,
    FixupMatches,
    MembershipHistory,
    MapHistory { format: String },
}

const LAST_MATCH: u32 = 117047;
const MAYBE_FIRST_MATCH: u32 = 14486;

#[tokio::main]
async fn main() -> MainResult {
    tracing_subscriber::fmt::init();
    let args = Args::parse();
    let config = Config::read(&args.config)?;
    let client = UgcClient::new(config.api.url);
    let archive = Archive::new(&config.db.url, &config.db.password()?).await?;

    match args.command {
        Command::Matches => {
            archive_matches(&client, &archive).await?;
        }
        Command::Teams => {
            archive_teams(&client, &archive).await?;
        }
        Command::FixupTeams => {
            fixup_teams(&client, &archive).await?;
        }
        Command::FixupMatches => {
            fixup_matches(&client, &archive).await?;
        }
        Command::MembershipHistory => {
            archive_team_roster_history(&client, &archive).await?;
        }
        Command::Players => {
            archive_players(&client, &archive).await?;
        }
        Command::MapHistory { format } => {
            let format = GameMode::from_str(&format)?;
            archive_map_history(&client, &archive, format).await?;
        }
    }
    Ok(())
}

async fn archive_matches(client: &UgcClient, archive: &Archive) -> MainResult {
    let next_match = archive
        .get_last_match_id()
        .await?
        .unwrap_or(MAYBE_FIRST_MATCH - 1)
        + 1;
    for id in next_match..=LAST_MATCH {
        let _span = span!(Level::INFO, "archive_match", id = id).entered();
        match client.get_match(id).await.check_not_found() {
            Ok(Some(match_data)) => {
                info!("storing match");
                archive.store_match(id, match_data).await?;
            }
            Ok(None) => {
                warn!("match not found");
            }
            Err(e) => {
                error!("error fetching match: {}", e);
            }
        }
        sleep(Duration::from_millis(500)).await;
    }
    Ok(())
}

async fn archive_teams(client: &UgcClient, archive: &Archive) -> MainResult {
    let range = archive.get_team_range().await?;
    let next_team = archive.get_last_team_id().await?.unwrap_or(range.start - 1) + 1;

    for id in next_team..=range.end {
        let _span = span!(Level::INFO, "archive_team", id = id).entered();
        match client.get_team(id).await.check_not_found() {
            Ok(Some(team_data)) => {
                if team_data.format.is_tf2() {
                    info!("storing team");
                    archive.store_team(id, &team_data).await?;
                } else {
                    info!("skipping non-tf2 team");
                }
            }
            Ok(None) => {
                warn!("team not found");
            }
            Err(e) => {
                error!("error fetching team: {:?}", e);
                panic!();
            }
        }
        sleep(Duration::from_millis(500)).await;
    }
    Ok(())
}

async fn archive_team_roster_history(client: &UgcClient, archive: &Archive) -> MainResult {
    let last = archive.get_max_roster_history().await?;
    let mut ids = pin!(archive.get_team_ids(last));

    while let Some(Ok(id)) = ids.next().await {
        let _span = span!(Level::INFO, "archive_team_roster_history", id = id).entered();
        match client.get_team_roster(id).await.check_not_found() {
            Ok(Some(team_data)) => {
                info!(count = team_data.len(), "storing team roster history");
                archive.store_membership_history(id, &team_data).await?;
            }
            Ok(None) => {
                warn!("team roster history not found");
            }
            Err(e) => {
                error!("error fetching team roster history: {:?}", e);
                panic!();
            }
        }
        sleep(Duration::from_millis(500)).await;
    }
    Ok(())
}

async fn fixup_teams(client: &UgcClient, archive: &Archive) -> MainResult {
    let mut ids = pin!(archive.get_no_region_teams());

    while let Some(Ok(id)) = ids.next().await {
        let _span = span!(Level::INFO, "fixup_team", id = id).entered();
        match client.get_team(id).await.check_not_found() {
            Ok(Some(team_data)) => {
                if team_data.format.is_tf2() {
                    info!(region = ?team_data.region, "updating team region");
                    archive.update_team_region(id, &team_data).await?;
                } else {
                    info!("skipping non-tf2 team");
                }
            }
            Ok(None) => {
                warn!("team not found");
            }
            Err(e) => {
                error!("error fetching team: {:?}", e);
                panic!();
            }
        }
        sleep(Duration::from_millis(500)).await;
    }
    Ok(())
}

async fn archive_players(client: &UgcClient, archive: &Archive) -> MainResult {
    let last = archive.get_max_player().await?;
    let mut ids = pin!(archive.get_players_ids(last));

    while let Some(Ok(steam_id)) = ids.next().await {
        let _span = span!(
            Level::INFO,
            "archive_player",
            steam_id = u64::from(steam_id)
        )
        .entered();
        match client.get_player(steam_id).await.check_not_found() {
            Ok(Some(player)) => {
                info!("storing player");
                archive.store_player(player).await?;
                // panic!();
            }
            Ok(None) => {
                warn!("player not found");
            }
            Err(e) => {
                error!("error fetching player: {:?}", e);
                panic!();
            }
        }
        sleep(Duration::from_millis(500)).await;
    }
    Ok(())
}

async fn archive_map_history(client: &UgcClient, archive: &Archive, mode: GameMode) -> MainResult {
    let history = client.get_maps(mode).await?;
    archive.store_map_history(mode, &history).await?;

    Ok(())
}

async fn fixup_matches(client: &UgcClient, archive: &Archive) -> MainResult {
    let mut match_ids = pin!(archive.get_match_ids_without_map());

    while let Some(Ok(id)) = match_ids.next().await {
        let _span = span!(Level::INFO, "fixup_match", id = id).entered();
        let match_info = client.get_match(id).await?;
        let date = archive.get_match_date(&match_info).await?;
        if date.is_none()
            && (match_info.format == GameMode::Highlander
                || match_info.format == GameMode::Sixes
                || match_info.format == GameMode::Fours
                || match_info.format == GameMode::Ultiduo)
        {
            dbg!(match_info.default_date);
            error!("failed to parse match date");
            panic!();
        }
        info!(date = ?date, format = %match_info.format, "updating match");
        archive.update_match_details(id, &match_info, date).await?;
        sleep(Duration::from_millis(500)).await;
    }
    Ok(())
}

trait NotFoundResultExt<T>: Sized {
    fn check_not_found(self) -> Result<Option<T>, UgcClientError>;
}

impl<T> NotFoundResultExt<T> for Result<T, UgcClientError> {
    fn check_not_found(self) -> Result<Option<T>, UgcClientError> {
        match self {
            Ok(x) => Ok(Some(x)),
            Err(UgcClientError::NotFound { .. }) => Ok(None),
            Err(e) => Err(e),
        }
    }
}
