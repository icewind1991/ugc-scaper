mod archive;
mod client;
mod config;

use crate::archive::Archive;
use crate::client::{UgcClient, UgcClientError};
use crate::config::Config;
use clap::ValueEnum;
use clap::{Parser, Subcommand};
use main_error::MainResult;
use std::fs::OpenOptions;
use std::io::BufWriter;
use std::io::Write;
use std::path::PathBuf;
use std::pin::pin;
use std::str::FromStr;
use std::time::Duration;
use tokio::time::sleep;
use tokio_stream::StreamExt;
use tracing::{error, info, instrument, span, warn, Level};
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
    Dump { data: Data, target: String },
}

#[derive(Debug, ValueEnum, Clone, Copy, Eq, PartialEq)]
enum Data {
    Teams,
    Players,
    Matches,
    Membership,
}

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
        Command::Dump { data, target } => {
            dump_data(&archive, data, &target).await?;
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
    for id in next_match..=MAYBE_FIRST_MATCH {
        archive_match(client, archive, id).await.ok();
        sleep(Duration::from_millis(500)).await;
    }
    Ok(())
}

#[instrument(skip(client, archive))]
async fn archive_match(client: &UgcClient, archive: &Archive, id: u32) -> MainResult {
    match client.get_match(id).await.check_not_found() {
        Ok(Some(match_data)) => {
            info!("storing match");
            archive.store_match(id as i32, match_data).await?;
            Ok(())
        }
        Ok(None) => {
            warn!("match not found");
            Ok(())
        }
        Err(e) => {
            error!("error fetching match: {}", e);
            Err(e.into())
        }
    }
}

async fn archive_teams(client: &UgcClient, archive: &Archive) -> MainResult {
    let range = archive.get_team_range().await?;
    let next_team = archive.get_last_team_id().await?.unwrap_or(range.start - 1) + 1;

    for id in next_team..=range.end {
        archive_team(client, archive, id).await?;
        sleep(Duration::from_millis(500)).await;
    }
    Ok(())
}

#[instrument(skip(client, archive))]
async fn archive_team(client: &UgcClient, archive: &Archive, id: u32) -> MainResult {
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
    let min_team = archive.get_min_team_id_without_match_seasons().await?;
    if min_team > 0 {
        let mut team_ids = pin!(archive.get_team_ids(min_team - 1));

        while let Some(Ok(team_id)) = team_ids.next().await {
            let _span = span!(Level::INFO, "fixup_matches", team_id).entered();
            let format = archive.get_team_format(team_id).await?;
            let matches = client.get_team_matches(team_id).await?;
            info!(
                seasons = matches.seasons.len(),
                ?format,
                "updating matches for team"
            );

            for season in matches.seasons.iter() {
                for season_match in season.matches.iter() {
                    if let Some(match_id) = season_match.result.match_id() {
                        if !archive.has_match(match_id).await? {
                            warn!(match_id, "match not archived yet");
                            sleep(Duration::from_millis(500)).await;
                            if let Err(_) = archive_match(client, archive, match_id).await {
                                let match_info = season_match
                                    .match_info(&matches.team, season.format)
                                    .expect("failed to build match info");
                                assert_eq!(format, match_info.format);
                                info!("reconstructed match");
                                archive.store_match(match_id as i32, match_info).await?;
                            }
                        }
                    }
                }

                archive
                    .update_match_details_from_team_matches(&matches.team, format, season)
                    .await?;
            }

            sleep(Duration::from_millis(500)).await;
        }
    }

    let mut last_team_id = 0;
    while let Some(team_id) = archive.get_min_team_id_without_default_date().await? {
        if team_id == last_team_id {
            panic!("team didn't get fixed up");
        }
        last_team_id = team_id;

        let _span = span!(Level::INFO, "fixup_matches", team_id).entered();
        let format = archive.get_team_format(team_id).await?;
        let matches = client.get_team_matches(team_id).await?;
        info!(
            seasons = matches.seasons.len(),
            ?format,
            "updating match date for team"
        );

        for season in matches.seasons.iter() {
            archive
                .update_match_date_from_team_matches(format, season)
                .await?;
        }

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

async fn dump_data(archive: &Archive, data: Data, output: &str) -> MainResult {
    let mut output = BufWriter::new(
        OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(output)?,
    );
    writeln!(&mut output, "[")?;
    let mut first = true;
    match data {
        Data::Teams => {
            let mut teams = pin!(archive.get_teams());
            while let Some(res) = teams.next().await {
                let team = res?;
                if !first {
                    writeln!(&mut output, ",")?;
                }
                first = false;
                write!(&mut output, "\t")?;
                serde_json::to_writer(&mut output, &team)?;
            }
        }
        Data::Players => {
            let mut players = pin!(archive.get_players());
            while let Some(res) = players.next().await {
                let player = res?;
                if !first {
                    writeln!(&mut output, ",")?;
                }
                first = false;
                write!(&mut output, "\t")?;
                serde_json::to_writer(&mut output, &player)?;
            }
        }
        Data::Matches => {
            let mut matches = pin!(archive.get_matches());
            while let Some(res) = matches.next().await {
                let matches = res?;
                if !first {
                    writeln!(&mut output, ",")?;
                }
                first = false;
                write!(&mut output, "\t")?;
                serde_json::to_writer(&mut output, &matches)?;
            }
        }
        Data::Membership => {
            let mut memberships = pin!(archive.get_membership());
            while let Some(res) = memberships.next().await {
                let membership = res?;
                if !first {
                    writeln!(&mut output, ",")?;
                }
                first = false;
                write!(&mut output, "\t")?;
                serde_json::to_writer(&mut output, &membership)?;
            }
        }
    }
    writeln!(&mut output, "\n]")?;
    Ok(())
}
