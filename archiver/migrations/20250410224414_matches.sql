CREATE TABLE IF NOT EXISTS matches
(
    id             INTEGER  NOT NULL,
    team_home      INTEGER  NOT NULL,
    team_away      INTEGER  NOT NULL,
    score_home     SMALLINT NOT NULL,
    score_away     SMALLINT NOT NULL,
    comment        VARCHAR,
    comment_author VARCHAR
);

CREATE UNIQUE INDEX matches_id_idx
    ON matches USING BTREE (id);

CREATE INDEX matches_home_idx
    ON matches USING BTREE (team_home);

CREATE INDEX matches_away_idx
    ON matches USING BTREE (team_away);