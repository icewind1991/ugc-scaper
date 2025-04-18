CREATE TABLE maps
(
    format game_mode NOT NULL,
    season INT       NOT NULL,
    week   INT       NOT NULL,
    date   DATE      NOT NULL,
    map    VARCHAR
);

CREATE UNIQUE INDEX maps_format_season_week_idx
    ON maps USING BTREE (format, season, week);

CREATE INDEX maps_map_idx
    ON maps USING BTREE (map);