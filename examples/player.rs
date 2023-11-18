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
    println!("{}", player.name);
    for team in player.teams {
        println!(
            "  {} playing {} since {}",
            team.team.name, team.league, team.since
        )
    }

    println!();
    println!("previous teams:");
    let membership = client.player_team_history(id).await?;
    for team in membership {
        if let Some(left) = team.left {
            println!(
                "  {} in {} from {} till {}",
                team.team.name, team.division, team.joined, left
            );
        }
    }

    Ok(())
}
