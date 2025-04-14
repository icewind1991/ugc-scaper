#[cfg(feature = "serde")]
use insta::assert_json_snapshot;
#[cfg(feature = "serde")]
use std::fs::read_to_string;
#[cfg(feature = "serde")]
use test_case::test_case;
#[cfg(feature = "serde")]
use ugc_scraper::parser::{
    MapHistoryParser, MatchPageParser, Parser, PlayerDetailsParser, PlayerParser, SeasonsParser,
    TeamLookupParser, TeamMatchesParser, TeamParser, TeamRosterHistoryParser, TransactionParser,
};

#[test_case("player_76561198024494988.html", "player")]
#[test_case("player_76561198049312442.html", "player_classes")]
#[cfg(feature = "serde")]
fn test_parse_player_html(input: &str, name: &str) {
    let body = read_to_string(format!("tests/data/{input}")).unwrap();
    let parser = PlayerParser::new();
    let parsed = parser.parse(&body).unwrap();
    assert_json_snapshot!(format!("parse_{name}_html"), parsed);
}

#[test_case("player_details_76561198024494988.html", "player_details")]
#[cfg(feature = "serde")]
fn test_parse_player_details_html(input: &str, name: &str) {
    let body = read_to_string(format!("tests/data/{input}")).unwrap();
    let parser = PlayerDetailsParser::new();
    let parsed = parser.parse(&body).unwrap();
    assert_json_snapshot!(format!("parse_{name}_html"), parsed);
}

#[test_case("team_7861.html", "team")]
#[test_case("team_4105.html", "older_team")]
#[test_case("team_8157.html", "team_no_tz")]
#[test_case("team_6929.html", "team_changed_name")]
#[test_case("team_32437.html", "team_empty_name_change")]
#[test_case("team_29228.html", "team_newlines_join_date")]
#[test_case("team_10763.html", "team_na_4v4")]
#[cfg(feature = "serde")]
fn test_parse_team_html(input: &str, name: &str) {
    let body = read_to_string(format!("tests/data/{input}")).unwrap();
    let parser = TeamParser::new();
    let parsed = parser.parse(&body).unwrap();
    assert_json_snapshot!(format!("parse_{name}_html"), parsed);
}

#[test_case("team_roster_history_7861.html", "team_roster_history")]
#[test_case("team_roster_history_2133.html", "team_roster_history_no_group")]
#[cfg(feature = "serde")]
fn test_parse_team_roster_history_html(input: &str, name: &str) {
    let body = read_to_string(format!("tests/data/{input}")).unwrap();
    let parser = TeamRosterHistoryParser::new();
    let parsed = parser.parse(&body).unwrap();
    assert_json_snapshot!(format!("parse_{name}_html"), parsed);
}

#[test_case("team_matches_7861.html", "team_matches")]
#[cfg(feature = "serde")]
fn test_parse_team_matches_html(input: &str, name: &str) {
    let body = read_to_string(format!("tests/data/{input}")).unwrap();
    let parser = TeamMatchesParser::new();
    let parsed = parser.parse(&body).unwrap();
    assert_json_snapshot!(format!("parse_{name}_html"), parsed);
}

#[test_case("index.html", "seasons")]
#[cfg(feature = "serde")]
fn test_parse_seasons_html(input: &str, name: &str) {
    let body = read_to_string(format!("tests/data/{input}")).unwrap();
    let parser = SeasonsParser::new();
    let parsed = parser.parse(&body).unwrap();
    assert_json_snapshot!(format!("parse_{name}_html"), parsed);
}

#[test_case("lookup_9v9.html", "seasons_9")]
#[test_case("lookup_6v6.html", "seasons_6")]
#[test_case("lookup_4v4.html", "seasons_4")]
#[test_case("lookup_2v2.html", "seasons_2")]
#[cfg(feature = "serde")]
fn test_parse_seasons_mode_html(input: &str, name: &str) {
    let body = read_to_string(format!("tests/data/{input}")).unwrap();
    let parser = TeamLookupParser::new();
    let parsed = parser.parse(&body).unwrap();
    assert_json_snapshot!(format!("parse_{name}_html"), parsed);
}

#[test_case("match_116246.html", "match")]
#[cfg(feature = "serde")]
fn test_parse_match_html(input: &str, name: &str) {
    let body = read_to_string(format!("tests/data/{input}")).unwrap();
    let parser = MatchPageParser::new();
    let parsed = parser.parse(&body).unwrap();
    assert_json_snapshot!(format!("parse_{name}_html"), parsed);
}

#[test_case("transactions_4v4.html", "transaction")]
#[cfg(feature = "serde")]
fn test_parse_transaction_html(input: &str, name: &str) {
    let body = read_to_string(format!("tests/data/{input}")).unwrap();
    let parser = TransactionParser::new();
    let parsed = parser.parse(&body).unwrap();
    assert_json_snapshot!(format!("parse_{name}_html"), parsed);
}

#[test_case("map_9v9.html", "maps_9")]
#[test_case("map_9v9_2.html", "maps_9_2")]
#[test_case("map_6v6.html", "maps_6")]
#[test_case("map_4v4.html", "maps_4")]
#[test_case("map_2v2.html", "maps_2")]
#[cfg(feature = "serde")]
fn test_parse_maps_html(input: &str, name: &str) {
    let body = read_to_string(format!("tests/data/{input}")).unwrap();
    let parser = MapHistoryParser::new();
    let parsed = parser.parse(&body).unwrap();
    assert_json_snapshot!(format!("parse_{name}_html"), parsed);
}
