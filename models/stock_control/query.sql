SELECT [time], boiler_temp_c, grouphead_temp_c, heat_level
FROM measurement
WHERE [time] > 1692712800000 AND [time] < 1692716400000
