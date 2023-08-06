CREATE TABLE IF NOT EXISTS measurement (
    time INTEGER PRIMARY KEY NOT NULL,

    target_temp_c FLOAT NOT NULL,

    boiler_temp_c FLOAT NOT NULL,
    grouphead_temp_c FLOAT NOT NULL,
    thermofilter_temp_c FLOAT NULL,

    -- Indicates the machine power
    power BOOLEAN NOT NULL,

    -- Indicates boiler power
    heat BOOLEAN NOT NULL,

    -- Indicates whether a shot is being pulled
    pull BOOLEAN NOT NULL,

    -- Indicates whether the
    steam BOOLEAN NOT NULL
);
