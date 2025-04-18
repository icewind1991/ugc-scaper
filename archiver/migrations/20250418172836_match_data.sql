ALTER TABLE matches
    ADD COLUMN IF NOT EXISTS map          VARCHAR,
    ADD COLUMN IF NOT EXISTS week         INT,
    ADD COLUMN IF NOT EXISTS format       game_mode,
    ADD COLUMN IF NOT EXISTS default_data DATE;