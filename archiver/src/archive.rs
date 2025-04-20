use futures_util::stream::TryStreamExt;
use sqlx::postgres::PgConnectOptions;
use sqlx::{query, Error, Executor, PgPool, Postgres};
use std::ops::Range;
use std::str::FromStr;
use thiserror::Error;
use time::format_description::FormatItem;
use time::macros::format_description;
use time::parsing::Parsed;
use time::{Date, Duration};
use tokio_stream::Stream;
use tracing::debug;
use ugc_scraper_types::{
    Class, GameMode, MapHistory, MatchInfo, Membership, MembershipRole, NameChange, Player, Record,
    Region, RosterHistory, SteamID, Team, TeamSeason, TeamSeasonMatch,
};

const MATCH_DATE_FORMAT: &[FormatItem<'static>] = format_description!(
    "[weekday case_sensitive:false repr:short], [month repr:short] [day padding:none] [year]"
);
const MATCH_DATE_FORMAT2: &[FormatItem<'static>] = format_description!(
    "[weekday case_sensitive:false repr:short] [month repr:short] [day padding:none] [year]"
);
const MATCH_DATE_FORMATS: &[&[FormatItem<'static>]] = &[MATCH_DATE_FORMAT, MATCH_DATE_FORMAT2];

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

    pub fn get_match_ids_without_map(
        &self,
    ) -> impl Stream<Item = Result<u32, ArchiveError>> + use<'_> {
        query!("select id from matches where map IS NULL ORDER BY id ASC")
            .fetch(&self.pool)
            .map_err(|error| ArchiveError::Query {
                description: "getting match ids",
                error,
            })
            .map_ok(|map| map.id as u32)
    }

    pub async fn get_match_date(
        &self,
        match_info: &MatchInfo,
    ) -> Result<Option<Date>, ArchiveError> {
        if let Ok(match_date) = parse_old_match_date(&match_info.default_date) {
            return Ok(Some(match_date));
        } else if !match_info.format.is_tf2() {
            return Ok(None);
        }

        let options = query!(
            "SELECT date FROM maps WHERE format = $1 AND week = $2 AND map = $3",
            match_info.format as GameMode,
            match_info.week as i32,
            match_info.map,
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|error| ArchiveError::Query {
            description: "searching map history",
            error,
        })?;

        for row in options {
            let possible_date: Date = row.date;
            let Some(match_date) = try_date_formats(
                match_info.default_date.as_str(),
                possible_date.year(),
                MATCH_DATE_FORMATS,
            ) else {
                break;
            };

            if (match_date - possible_date).abs() < Duration::days(7) {
                return Ok(Some(possible_date));
            }
        }

        Ok(None)
    }

    pub async fn update_match_details(
        &self,
        id: u32,
        match_info: &MatchInfo,
        date: Option<Date>,
    ) -> Result<(), ArchiveError> {
        let format = match_info.format.is_tf2().then_some(match_info.format);
        if let Some(date) = date {
            query!(
                "UPDATE matches SET map = $2, week = $3, format = $4, default_date = $5 WHERE id = $1",
                id as i32,
                match_info.map,
                match_info.week as i32,
                format as Option<GameMode>,
                date
            )
                .execute(&self.pool)
                .await
                .map_err(|error| ArchiveError::Query {
                    description: "updating match",
                    error,
                })?;
        } else {
            query!(
                "UPDATE matches SET map = $2, week = $3, format = $4 WHERE id = $1",
                id as i32,
                match_info.map,
                match_info.week as i32,
                format as Option<GameMode>,
            )
            .execute(&self.pool)
            .await
            .map_err(|error| ArchiveError::Query {
                description: "updating match",
                error,
            })?;
        }
        Ok(())
    }

    pub async fn update_match_details_from_team_matches(
        &self,
        season: TeamSeason,
    ) -> Result<(), ArchiveError> {
        let mut transaction = self
            .pool
            .begin()
            .await
            .map_err(|error| ArchiveError::Query {
                description: "beginning team matches transaction",
                error,
            })?;

        for match_info in season.matches {
            if let Some(id) = match_info.result.match_id() {
                query!(
                    "UPDATE matches SET map = $2, week = $3, format = $4, season = $5 WHERE id = $1",
                    id as i32,
                    match_info.map,
                    match_info.week as i32,
                    format as Option<GameMode>,
                    date
                )
                    .execute(&self.pool)
                    .await
                    .map_err(|error| ArchiveError::Query {
                        description: "updating match",
                        error,
                    })?;
            }
        }

        transaction
            .commit()
            .await
            .map_err(|error| ArchiveError::Query {
                description: "commiting team matches transaction",
                error,
            })?;

        Ok(())
    }
}

fn parse_old_match_date(date: &str) -> Result<Date, time::Error> {
    const MATCH_DATE_FORMAT_OLD: &[FormatItem<'static>] = format_description!("[weekday case_sensitive:false repr:short], [month padding:none]/[day padding:none]/[year repr:last_two]");
    let mut parsed = Parsed::new();
    parsed.parse_items(date.as_bytes(), MATCH_DATE_FORMAT_OLD)?;

    let year = parsed.year_last_two().unwrap() as i32 + 2000;
    parsed.set_year(year);
    Ok(Date::try_from(parsed)?)
}

#[test]
fn test_parse_old_match_date() {
    assert_eq!(
        Date::from_calendar_date(2009, time::Month::May, 13).unwrap(),
        parse_old_match_date("Wed, 5/13/09").unwrap()
    );
}

fn try_date_formats(date: &str, year: i32, formats: &[&[FormatItem<'static>]]) -> Option<Date> {
    for format in formats {
        match Date::parse(&format!("{} {}", date, year), format) {
            Ok(match_date) => {
                return Some(match_date);
            }
            Err(e) => {
                debug!(error = ?e, year, date, "date format not matching");
            }
        };
    }
    None
}

#[test]
fn test_parse_date() {
    assert!(try_date_formats("Sun Oct 06", 2019, MATCH_DATE_FORMATS).is_some());
}
