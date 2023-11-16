use main_error::MainResult;
use std::env::args;
use steamid_ng::SteamID;
use ugc_scraper::UgcClient;

#[tokio::main]
async fn main() -> MainResult {
    let client = UgcClient::new();
    let id = args().nth(1).expect("no steam id provided");
    let id = SteamID::try_from(id.as_str()).expect("invalid steam id provided");
    let player = client.player(id).await?;
    dbg!(player.teams);

    let membership = client.player_team_history(id).await?;
    dbg!(membership);

    Ok(())
}
