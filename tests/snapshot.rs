use insta::assert_json_snapshot;
use std::fs::read_to_string;
use ugc_scraper::parser::{
    MatchPageParser, Parser, PlayerDetailsParser, PlayerParser, SeasonsParser, TeamLookupParser,
    TeamMatchesParser, TeamParser, TeamRosterHistoryParser, TransactionParser,
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

#[test]
fn test_parse_seasons_html() {
    let body = read_to_string("tests/data/index.html").unwrap();
    let parser = SeasonsParser::new();
    let parsed = parser.parse(&body).unwrap();
    assert_json_snapshot!(parsed);
}

#[test]
fn test_parse_seasons_9_html() {
    let body = read_to_string("tests/data/lookup_9v9.html").unwrap();
    let parser = TeamLookupParser::new();
    let parsed = parser.parse(&body).unwrap();
    assert_json_snapshot!(parsed);
}

#[test]
fn test_parse_seasons_6_html() {
    let body = read_to_string("tests/data/lookup_6v6.html").unwrap();
    let parser = TeamLookupParser::new();
    let parsed = parser.parse(&body).unwrap();
    assert_json_snapshot!(parsed);
}

#[test]
fn test_parse_seasons_4_html() {
    let body = read_to_string("tests/data/lookup_4v4.html").unwrap();
    let parser = TeamLookupParser::new();
    let parsed = parser.parse(&body).unwrap();
    assert_json_snapshot!(parsed);
}

#[test]
fn test_parse_seasons_2_html() {
    let body = read_to_string("tests/data/lookup_2v2.html").unwrap();
    let parser = TeamLookupParser::new();
    let parsed = parser.parse(&body).unwrap();
    assert_json_snapshot!(parsed);
}

#[test]
fn test_parse_match_html() {
    let body = read_to_string("tests/data/match_116246.html").unwrap();
    let parser = MatchPageParser::new();
    let parsed = parser.parse(&body).unwrap();
    assert_json_snapshot!(parsed);
}

#[test]
fn test_parse_transaction_html() {
    let body = read_to_string("tests/data/transactions_4v4.html").unwrap();
    let parser = TransactionParser::new();
    let parsed = parser.parse(&body).unwrap();
    assert_json_snapshot!(parsed);
}
