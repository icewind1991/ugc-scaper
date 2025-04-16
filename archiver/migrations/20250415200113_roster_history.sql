CREATE TABLE membership_history
(
    team_id  INTEGER         NOT NULL,
    steam_id BIGINT          NOT NULL,
    role     membership_role NOT NULL,
    joined   DATE            NOT NULL,
    "left"   DATE
);

CREATE INDEX membership_history_team_id_idx
    ON membership_history USING BTREE (team_id);

CREATE INDEX membership_history_steam_id_idx
    ON membership_history USING BTREE (steam_id);