use sqlx::postgres::PgConnectOptions;
use sqlx::{query, Error, PgPool};
use std::str::FromStr;
use thiserror::Error;
use ugc_scraper_types::MatchInfo;

#[derive(Debug, Error)]
pub enum ArchiveError {
    #[error("Invalid database {url}: {error:#}")]
    InvalidDbUrl { url: String, error: Error },
    #[error("Error while connecting to database {url}: {error:#}")]
    Connect { url: String, error: sqlx::Error },
    #[error("Error while running query for {description}: {error:#}")]
    Query {
        description: &'static str,
        error: sqlx::Error,
    },
}

pub struct Archive {
    pool: PgPool,
}

impl Archive {
    pub async fn new(url: &str, password: &str) -> Result<Archive, ArchiveError> {
        let opt = PgConnectOptions::from_str(url)
            .map_err(|error| ArchiveError::InvalidDbUrl {
                url: url.into(),
                error,
            })?
            .password(password);
        let pool = PgPool::connect_with(opt)
            .await
            .map_err(|error| ArchiveError::Connect {
                url: url.into(),
                error,
            })?;
        Ok(Archive { pool })
    }

    pub async fn store_match(&self, id: u32, match_info: MatchInfo) -> Result<(), ArchiveError> {
        query!(
            "INSERT INTO matches (
                id, team_home, team_away, score_home, score_away, comment, comment_author
              ) VALUES ($1, $2, $3, $4, $5, $6, $7)",
            id as i32,
            match_info.team_home.id as i32,
            match_info.team_away.id as i32,
            match_info.score_home as i16,
            match_info.score_away as i16,
            match_info.comment,
            match_info.comment_author
        )
        .execute(&self.pool)
        .await
        .map_err(|error| ArchiveError::Query {
            description: "inserting match",
            error,
        })?;
        Ok(())
    }

    pub async fn get_last_match_id(&self) -> Result<Option<u32>, ArchiveError> {
        Ok(query!("SELECT id FROM matches ORDER BY id DESC LIMIT 1")
            .fetch_optional(&self.pool)
            .await
            .map_err(|error| ArchiveError::Query {
                description: "getting latest match",
                error,
            })?
            .map(|row| row.id as u32))
    }
}
