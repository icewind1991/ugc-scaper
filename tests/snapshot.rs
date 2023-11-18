use insta::assert_json_snapshot;
use std::fs::read_to_string;
use ugc_scraper::parser::{
    Parser, PlayerDetailsParser, PlayerParser, TeamMatchesParser, TeamParser,
    TeamRosterHistoryParser,
};

#[test]
fn test_parse_player_html() {
    let body = read_to_string("tests/data/player_76561198024494988.html").unwrap();
    let parser = PlayerParser::new();
    let parsed = parser.parse(&body).unwrap();
    assert_json_snapshot!(parsed);
}

#[test]
fn test_parse_player_details_html() {
    let body = read_to_string("tests/data/player_details_76561198024494988.html").unwrap();
    let parser = PlayerDetailsParser::new();
    let parsed = parser.parse(&body).unwrap();
    assert_json_snapshot!(parsed);
}

#[test]
fn test_parse_team_html() {
    let body = read_to_string("tests/data/team_7861.html").unwrap();
    let parser = TeamParser::new();
    let parsed = parser.parse(&body).unwrap();
    assert_json_snapshot!(parsed);
}

#[test]
fn test_parse_team_changed_name_html() {
    let body = read_to_string("tests/data/team_6929.html").unwrap();
    let parser = TeamParser::new();
    let parsed = parser.parse(&body).unwrap();
    assert_json_snapshot!(parsed);
}

#[test]
fn test_parse_team_roster_history_html() {
    let body = read_to_string("tests/data/team_roster_history_7861.html").unwrap();
    let parser = TeamRosterHistoryParser::new();
    let parsed = parser.parse(&body).unwrap();
    assert_json_snapshot!(parsed);
}

#[test]
fn test_parse_team_matches_html() {
    let body = read_to_string("tests/data/team_matches_7861.html").unwrap();
    let parser = TeamMatchesParser::new();
    let parsed = parser.parse(&body).unwrap();
    assert_json_snapshot!(parsed);
}
