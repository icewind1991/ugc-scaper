CREATE TYPE region AS ENUM ('europe', 'north-america', 'south-america', 'asia', 'australia');
CREATE TYPE game_mode AS ENUM ('highlander', 'eights', 'sixes', 'fours', 'ultiduo');
CREATE TYPE membership_role AS ENUM ('leader', 'member');

CREATE TABLE teams
(
    id          INTEGER   NOT NULL,
    name        VARCHAR   NOT NULL,
    image       VARCHAR   NOT NULL,
    format      game_mode NOT NULL,
    region      region,
    timezone    VARCHAR   NOT NULL,
    steam_group VARCHAR,
    division    VARCHAR   NOT NULL,
    description VARCHAR   NOT NULL
);

CREATE UNIQUE INDEX teams_id_idx
    ON teams USING BTREE (id);

CREATE INDEX teams_format_idx
    ON teams USING BTREE (format);

CREATE INDEX teams_region_idx
    ON teams USING BTREE (region);

CREATE INDEX teams_division_idx
    ON teams USING BTREE (division);

CREATE INDEX teams_timezone_idx
    ON teams USING BTREE (timezone);

CREATE TABLE titles
(
    team_id INTEGER NOT NULL,
    title   VARCHAR NOT NULL
);

CREATE INDEX titles_team_id_idx
    ON titles USING BTREE (team_id);

CREATE TABLE name_changes
(
    team_id   INTEGER NOT NULL,
    from_tag  VARCHAR NOT NULL,
    from_name VARCHAR NOT NULL,
    to_tag    VARCHAR NOT NULL,
    to_name   VARCHAR NOT NULL,
    date      DATE
);

CREATE INDEX name_changes_team_id_idx
    ON name_changes USING BTREE (team_id);

CREATE TABLE records
(
    team_id INTEGER NOT NULL,
    season  INTEGER NOT NULL,
    wins    INTEGER NOT NULL,
    losses  INTEGER NOT NULL
);

CREATE INDEX records_team_id_idx
    ON records USING BTREE (team_id);

CREATE INDEX records_season_idx
    ON records USING BTREE (season);

CREATE TABLE memberships
(
    team_id  INTEGER                  NOT NULL,
    steam_id BIGINT                   NOT NULL,
    role     membership_role          NOT NULL,
    since    TIMESTAMP WITH TIME ZONE NOT NULL
);

CREATE INDEX memberships_team_id_idx
    ON memberships USING BTREE (team_id);

CREATE INDEX memberships_steam_id_idx
    ON memberships USING BTREE (steam_id);