-- Find the modal grouphead temperature for each target temperature
-- The grouphead temperature is rounded to the nearest 2 degrees to account for fluctuations.
WITH RoundedTemps AS (
    SELECT
        target_temp_c,
        ROUND(grouphead_temp_c / 2) * 2 as rounded_grouphead_temp_c
    FROM measurement
    WHERE time > 1693170660000 -- Aug 27, 2023, 10:11 PM GMT+1
    AND power IS TRUE AND STEAM IS FALSE AND pull IS FALSE
),

TempCounts AS (
    SELECT
        target_temp_c,
        rounded_grouphead_temp_c,
        COUNT(*) as count
    FROM RoundedTemps
    GROUP BY target_temp_c, rounded_grouphead_temp_c
),

MaxCounts AS (
    SELECT
        target_temp_c,
        MAX(count) as max_count
    FROM TempCounts
    GROUP BY target_temp_c
)

SELECT
    m.target_temp_c,
    t.rounded_grouphead_temp_c as modal_grouphead_temp_c
FROM MaxCounts m
JOIN TempCounts t ON m.target_temp_c = t.target_temp_c AND m.max_count = t.count
ORDER BY m.target_temp_c;
