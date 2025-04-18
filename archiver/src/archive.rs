use futures_util::stream::TryStreamExt;
use sqlx::postgres::PgConnectOptions;
use sqlx::{query, Error, Executor, PgPool, Postgres};
use std::ops::Range;
use std::str::FromStr;
use thiserror::Error;
use tokio_stream::Stream;
use ugc_scraper_types::{
    Class, GameMode, MapHistory, MatchInfo, Membership, MembershipRole, NameChange, Player, Record,
    Region, RosterHistory, SteamID, Team,
};

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
    #[error("Error while parsing dates for {format}")]
    DateFormat { format: GameMode },
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

    pub async fn store_team(&self, id: u32, team: &Team) -> Result<(), ArchiveError> {
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|error| ArchiveError::Query {
                description: "beginning team transaction",
                error,
            })?;
        query!(
            "INSERT INTO teams (
                id, tag, name, image, format, region, timezone, steam_group, division, description
              ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)",
            id as i32,
            team.tag,
            team.name,
            team.image,
            team.format as GameMode,
            team.region as Option<Region>,
            team.timezone,
            team.steam_group,
            team.division,
            team.description,
        )
        .execute(&mut *transaction)
        .await
        .map_err(|error| ArchiveError::Query {
            description: "inserting team",
            error,
        })?;

        for title in team.titles.iter() {
            Self::store_title(&mut *transaction, id, title).await?;
        }
        for name_change in team.name_changes.iter() {
            Self::store_team_name_change(&mut *transaction, id, name_change).await?
        }
        for record in team.results.iter() {
            Self::store_record(&mut *transaction, id, record).await?
        }
        for membership in team.members.iter() {
            Self::store_membership(&mut *transaction, id, membership).await?
        }

        transaction
            .commit()
            .await
            .map_err(|error| ArchiveError::Query {
                description: "commiting team transaction",
                error,
            })?;
        Ok(())
    }

    pub async fn update_team_region(&self, id: u32, team: &Team) -> Result<(), ArchiveError> {
        query!(
            "UPDATE teams SET region = $2 WHERE id = $1",
            id as i32,
            team.region as Option<Region>,
        )
        .execute(&self.pool)
        .await
        .map_err(|error| ArchiveError::Query {
            description: "updating team region",
            error,
        })?;
        Ok(())
    }

    pub async fn get_last_team_id(&self) -> Result<Option<u32>, ArchiveError> {
        Ok(query!("SELECT id FROM teams ORDER BY id DESC LIMIT 1")
            .fetch_optional(&self.pool)
            .await
            .map_err(|error| ArchiveError::Query {
                description: "getting latest team",
                error,
            })?
            .map(|row| row.id as u32))
    }

    pub async fn get_team_range(&self) -> Result<Range<u32>, ArchiveError> {
        let row = query!("select greatest(max(team_home), max(team_away)) as max, least(min(team_home), min(team_away)) as min from matches limit 1;")
            .fetch_one(&self.pool)
            .await
            .map_err(|error| ArchiveError::Query {
                description: "getting latest team",
                error,
            })?;
        Ok((row.min.unwrap_or_default() as u32)..(row.max.unwrap_or_default() as u32))
    }

    pub fn get_team_ids(
        &self,
        min: u32,
    ) -> impl Stream<Item = Result<u32, ArchiveError>> + use<'_> {
        query!(
            "select id from teams where id > $1 order by id asc",
            min as i32
        )
        .fetch(&self.pool)
        .map_err(|error| ArchiveError::Query {
            description: "getting team ids",
            error,
        })
        .map_ok(|map| map.id as u32)
    }

    pub async fn get_max_roster_history(&self) -> Result<u32, ArchiveError> {
        if let Some(row) =
            query!("select team_id as max from membership_history order by team_id desc limit 1;")
                .fetch_optional(&self.pool)
                .await
                .map_err(|error| ArchiveError::Query {
                    description: "getting latest team membership history",
                    error,
                })?
        {
            Ok(row.max as u32)
        } else {
            Ok(0)
        }
    }

    pub fn get_players_ids(
        &self,
        min: SteamID,
    ) -> impl Stream<Item = Result<SteamID, ArchiveError>> + use<'_> {
        query!(
            "select distinct steam_id from membership_history where steam_id > $1 order by steam_id asc",
            u64::from(min) as i64,
        )
            .fetch(&self.pool)
            .map_err(|error| ArchiveError::Query {
                description: "getting player steam ids",
                error,
            })
            .map_ok(|map| (map.steam_id as u64).into())
    }

    pub async fn get_max_player(&self) -> Result<SteamID, ArchiveError> {
        if let Some(row) =
            query!("select steam_id as max from players order by steam_id desc limit 1;")
                .fetch_optional(&self.pool)
                .await
                .map_err(|error| ArchiveError::Query {
                    description: "getting latest team membership history",
                    error,
                })?
        {
            Ok((row.max as u64).into())
        } else {
            Ok(0.into())
        }
    }

    pub fn get_no_region_teams(&self) -> impl Stream<Item = Result<u32, ArchiveError>> + use<'_> {
        query!("select id from teams where region IS NULL and format != 'eights' order by id desc")
            .fetch(&self.pool)
            .map_err(|error| ArchiveError::Query {
                description: "getting teams without region",
                error,
            })
            .map_ok(|map| map.id as u32)
    }

    async fn store_title(
        db: impl Executor<'_, Database = Postgres>,
        team_id: u32,
        title: &str,
    ) -> Result<(), ArchiveError> {
        query!(
            "INSERT INTO titles (
                team_id, title
              ) VALUES ($1, $2)",
            team_id as i32,
            title
        )
        .execute(db)
        .await
        .map_err(|error| ArchiveError::Query {
            description: "inserting title",
            error,
        })?;
        Ok(())
    }

    async fn store_team_name_change(
        db: impl Executor<'_, Database = Postgres>,
        team_id: u32,
        change: &NameChange,
    ) -> Result<(), ArchiveError> {
        query!(
            "INSERT INTO team_name_changes (
                team_id, from_tag, from_name, to_tag, to_name, date
              ) VALUES ($1, $2, $3, $4, $5, $6)",
            team_id as i32,
            change.from_tag,
            change.from,
            change.to_tag,
            change.to,
            change.date
        )
        .execute(db)
        .await
        .map_err(|error| ArchiveError::Query {
            description: "inserting name change",
            error,
        })?;
        Ok(())
    }

    async fn store_membership(
        db: impl Executor<'_, Database = Postgres>,
        team_id: u32,
        membership: &Membership,
    ) -> Result<(), ArchiveError> {
        query!(
            "INSERT INTO memberships (
                team_id, steam_id, role, since
              ) VALUES ($1, $2, $3, $4)",
            team_id as i32,
            u64::from(membership.steam_id) as i64,
            membership.role as MembershipRole,
            membership.since,
        )
        .execute(db)
        .await
        .map_err(|error| ArchiveError::Query {
            description: "inserting membership",
            error,
        })?;
        Ok(())
    }

    async fn store_record(
        db: impl Executor<'_, Database = Postgres>,
        team_id: u32,
        record: &Record,
    ) -> Result<(), ArchiveError> {
        query!(
            "INSERT INTO records (
                team_id, season, wins, losses
              ) VALUES ($1, $2, $3, $4)",
            team_id as i32,
            record.season as i32,
            record.wins as i32,
            record.losses as i32,
        )
        .execute(db)
        .await
        .map_err(|error| ArchiveError::Query {
            description: "inserting record",
            error,
        })?;
        Ok(())
    }

    pub async fn store_membership_history(
        &self,
        team_id: u32,
        memberships: &[RosterHistory],
    ) -> Result<(), ArchiveError> {
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|error| ArchiveError::Query {
                description: "beginning membership history transaction",
                error,
            })?;

        for membership in memberships {
            query!(
                r#"INSERT INTO membership_history (
                team_id, steam_id, role, joined, "left"
              ) VALUES ($1, $2, $3, $4, $5)"#,
                team_id as i32,
                u64::from(membership.steam_id) as i64,
                membership.role as MembershipRole,
                membership.joined,
                membership.left,
            )
            .execute(&mut *transaction)
            .await
            .map_err(|error| ArchiveError::Query {
                description: "inserting membership history",
                error,
            })?;
        }

        transaction
            .commit()
            .await
            .map_err(|error| ArchiveError::Query {
                description: "commiting membership history transaction",
                error,
            })?;

        Ok(())
    }

    pub async fn store_player(&self, player: Player) -> Result<(), ArchiveError> {
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|error| ArchiveError::Query {
                description: "beginning player transaction",
                error,
            })?;

        query!(
            "INSERT INTO players (
                steam_id, name, avatar, favorite_classes, country
              ) VALUES ($1, $2, $3, $4, $5)",
            u64::from(player.steam_id) as i64,
            player.name,
            player.avatar,
            player.favorite_classes as Vec<Class>,
            player.country,
        )
        .execute(&mut *transaction)
        .await
        .map_err(|error| ArchiveError::Query {
            description: "inserting player",
            error,
        })?;

        for honors in player.honors.iter() {
            query!(
                "INSERT INTO player_honors (
                steam_id, team_id, season, division, format
              ) VALUES ($1, $2, $3, $4, $5)",
                u64::from(player.steam_id) as i64,
                honors.team.id as i32,
                honors.season as i16,
                honors.division,
                honors.format as GameMode,
            )
            .execute(&mut *transaction)
            .await
            .map_err(|error| ArchiveError::Query {
                description: "inserting player honors",
                error,
            })?;
        }

        transaction
            .commit()
            .await
            .map_err(|error| ArchiveError::Query {
                description: "commiting player transaction",
                error,
            })?;

        Ok(())
    }

    pub async fn store_map_history(
        &self,
        format: GameMode,
        maps: &MapHistory,
    ) -> Result<(), ArchiveError> {
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|error| ArchiveError::Query {
                description: "beginning map history transaction",
                error,
            })?;

        // who knows, the website doesn't say
        let current_season_year = 2024;

        for week in maps.weeks(current_season_year) {
            let week = week.map_err(|_| ArchiveError::DateFormat { format })?;
            query!(
                "INSERT INTO maps (
                    format, season, week, date, map
                  ) VALUES ($1, $2, $3, $4, $5)",
                format as GameMode,
                week.season as i32,
                week.week as i32,
                week.date,
                week.map,
            )
            .execute(&mut *transaction)
            .await
            .map_err(|error| ArchiveError::Query {
                description: "inserting map history",
                error,
            })?;
        }

        transaction
            .commit()
            .await
            .map_err(|error| ArchiveError::Query {
                description: "commiting map history transaction",
                error,
            })?;

        Ok(())
    }
}
