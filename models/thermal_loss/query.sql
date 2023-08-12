-- Get all the measurements where the machine is idle and the boiler temperature decreases over time
-- i.e. measurements whilst the machine is cooling down.
SELECT [time],
    boiler_temp_c,
    grouphead_temp_c
FROM (
        SELECT [time],
            LAG(boiler_temp_c) OVER () boiler_temp_c_prev,
            LAG([time]) OVER () time_prev,
            boiler_temp_c,
            grouphead_temp_c,
            LAG(grouphead_temp_c) OVER () grouphead_temp_c_prev,
            heat_level,
            pull,
            [power]
        from measurement
    ) AS inner_query
WHERE boiler_temp_c_prev >= boiler_temp_c
    AND grouphead_temp_c_prev >= grouphead_temp_c
    AND heat_level = 0.0
    AND pull = FALSE
    AND [power] = FALSE
ORDER BY [time] ASC;
