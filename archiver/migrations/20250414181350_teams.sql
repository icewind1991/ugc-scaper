CREATE TYPE region AS ENUM ('europe', 'north-america', 'south-america', 'asia', 'australia');
CREATE TYPE game_mode AS ENUM ('highlander', 'sixes', 'fours', 'ultiduo');

CREATE TABLE teams
(
    id             INTEGER  NOT NULL,
    name     VARCHAR NOT NULL,
    image      VARCHAR NOT NULL,
    form
);

CREATE UNIQUE INDEX matches_id_idx
    ON matches USING BTREE (id);

CREATE INDEX matches_home_idx
    ON matches USING BTREE (team_home);

CREATE INDEX matches_away_idx
    ON matches USING BTREE (team_away);