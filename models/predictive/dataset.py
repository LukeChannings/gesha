import json
import math
import sqlite3
from pathlib import Path
import sys
from typing import Any, List
from matplotlib.cbook import flatten
import numpy as np
import pandas as pd
from pandas import DataFrame, concat, read_sql_query, set_option, to_datetime
from tabulate import tabulate
from pre_heating.levels import get_preheat_level
from datetime import timedelta

set_option("display.max_rows", sys.maxsize)
set_option("display.max_columns", sys.maxsize)
set_option("display.width", 1000)

from util import get_git_root, strfdelta

root = Path(get_git_root())


def get_data():
    db = sqlite3.connect(root / "data/gesha-13-09-2023.db")
    df = read_sql_query(
        """
        SELECT time, target_temp_c, boiler_temp_c, grouphead_temp_c, heat_level, pull, power
        FROM measurement
        WHERE power IS TRUE
        ORDER BY time ASC
    """,
        db,
    )

    df["time"] = to_datetime(df["time"], unit="ms")

    return df


# The measurements will only be recorded if the temperature has changed since the last measurement.
# As such, we need to fill in the missing values with the last recorded value.
def upscale_data(df: DataFrame, period: Any = "1T"):
    return (
        df.set_index("time").resample(period).ffill().dropna(axis="index").reset_index()
    )

def group_by_contiguous_measurements(df: DataFrame):
    # Let's find any discontinuities in the temperatures to find non-continuous measurements.
    df["boiler_temp_c_diff"] = df["boiler_temp_c"].diff()

    df.dropna(inplace=True)

    # I analysed individual cases and found that a boiler temperature change of more than 2 degrees
    # indicates a non-continuous measurement
    df["group"] = (df["boiler_temp_c_diff"].abs() > 2.0).cumsum()

    groups = sorted(
        [g.iloc[1:] for _, g in df.groupby("group") if len(g) > 1000],
        key=len,
        reverse=True,
    )

    return groups


# Identify the beginning and end indices of each heat session
def get_heat_session_bounds(
    df: DataFrame,
    threshold: int = 0,
):
    indices = []

    heat_start_idx = None
    heat_end_idx = None

    # Initialise heat level to 0 so that we can detect the first heat session starting at index 0.
    prev_heat_level = 0

    for idx, heat_level in enumerate(df["heat_level"]):
        if prev_heat_level == 0 and heat_level > 0:
            if heat_start_idx is not None and heat_end_idx is not None:
                heat_end_idx = idx
            else:
                heat_start_idx = idx
                heat_end_idx = None
        elif prev_heat_level > 0 and heat_level == 0:
            heat_end_idx = idx
        elif prev_heat_level == 0 and heat_level == 0:
            if heat_end_idx is not None and idx - heat_end_idx >= threshold:
                indices.append((heat_start_idx, heat_end_idx))
                heat_start_idx = None
                heat_end_idx = None

        prev_heat_level = heat_level

    if heat_start_idx is not None and heat_end_idx is not None:
        indices.append((heat_start_idx, heat_end_idx))
    elif heat_start_idx is not None:
        indices.append((heat_start_idx, len(df) - 1))

    return indices


def get_heat_session_summaries(df: DataFrame, threshold, df_idx=0):
    indices = get_heat_session_bounds(df, threshold)

    column_names = [
        "session_duration",
        "heat_level_sum",
        "heat_level_avg",
        "temp_initial",
        "temp_initial_grouphead",
        "temp_heatoff",
        "temp_diff_heatoff",
        "temp_diff",
        "temp_plateau",
        "start_idx",
        "end_idx",
        "plateau_idx",
        "df_idx",
        "start_lag",
        "stop_lag",
        "max_1s_temp_diff",
    ]

    session_rows = []

    temp_diff_window = timedelta(seconds=1) // timedelta(milliseconds=100)
    df["boiler_temp_c_diff"] = df["boiler_temp_c"].rolling(window=temp_diff_window, min_periods=1).sum().diff().fillna(0)
    df["boiler_temp_increase"] = ((df["boiler_temp_c_diff"] - df["boiler_temp_c_diff"].shift(-1).fillna(0)) > 0.25).shift(0)

    for start_idx, end_idx in indices:
        plateau_idx = df.iloc[end_idx : end_idx + 3 * (end_idx - start_idx)][
            "boiler_temp_c"
        ].idxmax()
        temp_initial = df.iloc[start_idx]["boiler_temp_c"]
        temp_initial_grouphead = df.iloc[start_idx]["grouphead_temp_c"]
        temp_heatoff = df.iloc[end_idx]["boiler_temp_c"]
        temp_plateau = df.iloc[plateau_idx]["boiler_temp_c"]
        temp_diff = temp_plateau - temp_initial
        temp_diff_heatoff = temp_heatoff - temp_initial

        heat_level_sum = df.iloc[start_idx:end_idx]["heat_level"].sum()
        record_count = end_idx - start_idx
        stop_lag = (df.iloc[plateau_idx]["time"] - df.iloc[end_idx]["time"]).total_seconds()
        start_lag_idx = df["boiler_temp_increase"].iloc[start_idx:].idxmax()
        start_lag = (df.iloc[start_lag_idx]["time"] - df.iloc[start_idx]["time"]).total_seconds()
        max_1s_temp_diff = df["boiler_temp_c_diff"].iloc[start_idx:plateau_idx].max()

        if start_lag <= 0: continue

        session_rows.append(
            [
                (df.iloc[plateau_idx]["time"] - df.iloc[start_idx]["time"]).total_seconds(),
                heat_level_sum,
                (heat_level_sum / record_count),
                temp_initial,
                temp_initial_grouphead,
                temp_heatoff,
                temp_diff_heatoff,
                temp_diff,
                temp_plateau,
                start_idx,
                end_idx,
                plateau_idx,
                df_idx,
                start_lag,
                stop_lag,
                max_1s_temp_diff,
            ]
        )

    return DataFrame(session_rows, columns=column_names)

def get_heat_session_rows(df: DataFrame, start_idx, start_lag, stop_lag, plateau_idx):
    session_df = df.copy()

    lag_window = timedelta(seconds=start_lag) // timedelta(milliseconds=100)
    shift_window = timedelta(seconds=30) // timedelta(milliseconds=100)

    session_df["rolling_heat_level"] = session_df["heat_level"].rolling(window=lag_window, min_periods=1).sum().fillna(0)
    session_df["future_temp_diff"] = session_df["boiler_temp_c"].shift(-shift_window) - session_df["boiler_temp_c"]
    session_df = session_df.iloc[start_idx-lag_window:plateau_idx]
    session_df.dropna(inplace=True)

    return session_df


def get_nn_data():
    data = get_data()
    groups = group_by_contiguous_measurements(data)

    # The fixed time between measurements
    period = timedelta(milliseconds=100)

    heat_session_threshold = timedelta(seconds=20)

    result = []

    for df in groups:
        df = upscale_data(df, period="100ms")
        indices = get_heat_session_bounds(
            df, threshold=heat_session_threshold // period
        )
        df["rolling_heat_level"] = df["heat_level"].rolling(window=260, min_periods=1).sum().fillna(0)

        df["future_temp_diff"] = df["boiler_temp_c"].shift(-2160) - df["boiler_temp_c"]

        df.dropna(inplace=True)
        result.append(df)

    return pd.concat(result)


def get_correlation_optimised_measurement_data():
    data = get_data()
    groups = group_by_contiguous_measurements(data)
    # The fixed time between measurements
    period = timedelta(milliseconds=100)
    groups_upscaled = [upscale_data(df, period) for df in groups]

    result = []

    for i, df in enumerate(groups_upscaled):
        df["rolling_heat_level"] = df["heat_level"].rolling(window=260, min_periods=1).sum().fillna(0)

        df["future_temp_diff"] = df["boiler_temp_c"].shift(-2160)

        df.dropna(inplace=True)
        result.append(df)

    return pd.concat(result)


# Find the optimal coefficients of lookback window (n) and record shift count (m)
def search_best_corr(df: DataFrame):
    period = timedelta(milliseconds=100)
    n_range = [timedelta(seconds=s) // period for s in range(1, 16, 0.5)]
    m_range = [timedelta(seconds=s) // period for s in range(18, 48, 0.5)]

    scores = []

    for n in n_range:
        for m in m_range:
            df["rolling_heat_level"] = df["heat_level"].rolling(window=n).sum()

            df["boiler_temp_c_diff"] = df["boiler_temp_c"].diff()
            df["future_temp_diff"] = df["boiler_temp_c"].shift(-m)

            corr = df["rolling_heat_level"].corr(df["future_temp_diff"])

            scores.append((corr, (n, m)))

    return sorted(scores, key=lambda x: x[0], reverse=True)

