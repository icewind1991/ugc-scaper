use main_error::MainResult;
use ugc_scraper::UgcClient;

#[tokio::main]
async fn main() -> MainResult {
    let client = UgcClient::new();
    let modes = client.previous_seasons().await?;
    for mode in modes {
        println!("{}", mode.mode);

        for season in mode.seasons {
            println!("  {}: {}", season.id, season.name);
        }
    }

    Ok(())
}
