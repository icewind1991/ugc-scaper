use main_error::MainResult;
use std::env::args;
use ugc_scraper::UgcClient;

#[tokio::main]
async fn main() -> MainResult {
    let client = UgcClient::new();
    let id = args().nth(1).expect("no team id provided");
    let id = id.parse().expect("invalid team id provided");
    let team = client.team(id).await?;
    println!("{} - {}", team.tag, team.name);
    println!("playing {} in {}", team.format, team.division);
    println!();
    println!("with: ");
    for member in team.members {
        println!("  {} since {}", member.name, member.since);
    }

    println!();
    println!("previous players ");
    let roster_history = client.team_roster_history(id).await?;
    for roster_item in roster_history {
        if let Some(left) = roster_item.left {
            println!(
                "  {} joined at {} and left at {}",
                roster_item.name, roster_item.joined, left
            );
        }
    }

    println!();
    println!("name changes:");
    for name_change in team.name_changes {
        println!(
            "  {} - {} to {} - {} at {}",
            name_change.from_tag,
            name_change.from,
            name_change.to_tag,
            name_change.to,
            name_change.date
        );
    }

    Ok(())
}
