SELECT
    m.[time],
    s.start_time,
    s.end_time,
    m.boiler_temp_c,
    m.grouphead_temp_c,
    m.heat_level
FROM shot s
JOIN measurement m
    ON m.[time] >= s.start_time AND m.[time] <= s.end_time
WHERE
    s.start_time >= 1692128756946;
