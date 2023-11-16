use insta::assert_json_snapshot;
use std::fs::read_to_string;
use ugc_scraper::parser::{Parser, PlayerDetailsParser, PlayerParser};

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
