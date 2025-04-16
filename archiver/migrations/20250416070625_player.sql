CREATE TYPE player_class AS ENUM ('scout', 'soldier', 'pyro', 'demoman', 'engineer', 'heavy', 'medic', 'sniper', 'spy');

CREATE TABLE players
(
    steam_id         BIGINT         NOT NULL,
    name             VARCHAR        NOT NULL,
    avatar           VARCHAR,
    favorite_classes player_class[] NOT NULL
);

CREATE UNIQUE INDEX players_steam_id_idx
    ON players USING BTREE (steam_id);

CREATE TABLE player_honors
(
    steam_id BIGINT    NOT NULL,
    team_id  INT       NOT NULL,
    season   SMALLINT  NOT NULL,
    division VARCHAR   NOT NULL,
    format   game_mode NOT NULL
);

CREATE INDEX player_honors_steam_id_idx
    ON player_honors USING BTREE (steam_id);

CREATE INDEX player_honors_team_id_idx
    ON player_honors USING BTREE (team_id);

CREATE INDEX player_honors_season_idx
    ON player_honors USING BTREE (season);
