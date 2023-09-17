def get_preheat_level(target_temp, grouphead_temp):
    level = (
        grouphead_temp / 74.0 if target_temp <= 90.0 else
        grouphead_temp / 76.0 if target_temp <= 93.0 else
        grouphead_temp / 78.0 if target_temp <= 95.0 else
        grouphead_temp / 80.0 if target_temp <= 99.0 else
        grouphead_temp / 82.0 if target_temp <= 101.0 else
        grouphead_temp / 84.0 if target_temp <= 103.0 else
        grouphead_temp / 86.0 if target_temp <= 107.0 else
        grouphead_temp / 88.0 if target_temp <= 109.0 else
        grouphead_temp / 90.0
    )

    return min(level, 1.0)
