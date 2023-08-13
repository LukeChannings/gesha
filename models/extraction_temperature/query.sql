SELECT [time],
    boiler_temp_c,
    grouphead_temp_c,
    heat_level,
    pull
FROM measurement
WHERE [time] >= 1691527299412
    AND [time] <= 1691587236817;
