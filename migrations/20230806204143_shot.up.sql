CREATE TABLE IF NOT EXISTS shot (
    start_time INTEGER PRIMARY KEY NOT NULL,
    end_time INTEGER NOT NULL,
    total_time INTEGER NOT NULL,
    brew_temp_average_c FLOAT NOT NULL,
    grouphead_temp_avg_c FLOAT NOT NULL
);