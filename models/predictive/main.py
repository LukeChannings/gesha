from datetime import timedelta
import sys

import numpy as np
import pandas as pd
from predictive.dataset import (
    get_data,
    get_heat_session_rows,
    get_correlation_optimised_measurement_data,
    group_by_contiguous_measurements,
    get_heat_session_summaries,
    upscale_data,
)
from predictive.train_nn import (
    evaluate_simple_nn_1,
    train_heat_sum_model,
    train_simple_neural_network,
)
from util import CHARTS_PATH, MODELS_PATH, run_cli
from visualisation import graph_measurements

np.set_printoptions(threshold=sys.maxsize)


def get_processed_data():
    period = timedelta(milliseconds=100)
    threshold = timedelta(seconds=30) // period

    data_groups = group_by_contiguous_measurements(get_data())
    upscaled_data_groups = [upscale_data(group, period) for group in data_groups]
    session_summaries = pd.concat(
        [
            get_heat_session_summaries(group, threshold, df_idx)
            for df_idx, group in enumerate(upscaled_data_groups)
        ]
    )

    return session_summaries


def print_heat_sessions():
    heat_sessions = get_processed_data()

    pruned_heat_sessions = heat_sessions[
        ~(
            (heat_sessions["start_lag"] < 2)
            | (heat_sessions["start_lag"] > 20)
            | (heat_sessions["stop_lag"] < 25)
            | (heat_sessions["stop_lag"] > 90)
            | (heat_sessions["temp_initial"] > 100)
        )
    ]

    pruned_heat_sessions["heat_seconds"] = pruned_heat_sessions["heat_level_sum"] / 10

    print(
        pruned_heat_sessions[
            [
                "session_duration",
                "heat_seconds",
                "start_lag",
                "stop_lag",
                "temp_initial",
                "temp_diff",
            ]
        ].describe()
    )


def heat_sum_model():
    period = timedelta(milliseconds=100)
    threshold = (
        timedelta(seconds=7) // period
    )  # Found 7 seconds by optimising for the correlation between heat_level_sum and temp_diff.

    data_groups = group_by_contiguous_measurements(get_data())

    heat_session_summaries = pd.concat(
        [
            get_heat_session_summaries(upscale_data(g, period), threshold, i)
            for i, g in enumerate(data_groups)
        ]
    ).reset_index()

    heat_session_summaries.drop(
        index=heat_session_summaries[
            (heat_session_summaries["heat_level_sum"] > 1)
            & (heat_session_summaries["temp_diff"] < 1)
            & (heat_session_summaries["temp_initial"] > 99)
        ].index,
        inplace=True,
    )

    print(
        heat_session_summaries["heat_level_sum"].corr(
            heat_session_summaries["temp_diff"]
        )
    )

    train_heat_sum_model(heat_session_summaries)


def boiler_temp_diff_model():
    period = timedelta(milliseconds=100)
    threshold = (
        timedelta(seconds=7) // period
    )  # Found 7 seconds by optimising for the correlation between heat_level_sum and temp_diff.

    data_groups = group_by_contiguous_measurements(get_data())
    upscaled_data_groups = [upscale_data(group, period) for group in data_groups]
    session_summaries = pd.concat(
        [
            get_heat_session_summaries(group, threshold, df_idx)
            for df_idx, group in enumerate(upscaled_data_groups)
        ]
    )

    measurements = pd.concat(
        [
            get_heat_session_rows(upscaled_data_groups[df_idx], *row)
            for _, (
                *row,
                df_idx,
            ) in session_summaries[
                ["start_idx", "start_lag", "stop_lag", "plateau_idx", "df_idx"]
            ].iterrows()
        ]
    )

    print(
        measurements[
            [
                "grouphead_temp_c",
                "boiler_temp_c",
                "rolling_heat_level",
                "future_temp_diff",
            ]
        ].sort_values(by="future_temp_diff")[0:500]
    )

    train_simple_neural_network(
        measurements, str(MODELS_PATH / "predictive/output/subset_model.onnx")
    )


def main():
    run_cli(
        {
            "train": lambda: train_simple_neural_network(
                get_correlation_optimised_measurement_data(),
                str(MODELS_PATH / "predictive/output/predictive.onnx"),
            ),
            "train_subset": lambda: boiler_temp_diff_model(),
            "train_heat_sum": lambda: heat_sum_model(),
            "evaluate_original": lambda: evaluate_simple_nn_1(
                get_correlation_optimised_measurement_data()
            ),
            "print_heat_sessions": lambda: print_heat_sessions(),
            "graph_measurements": lambda: graph_measurements(
                1694431800000,
                1694442600000,
                save_path=str(CHARTS_PATH / "predictive-model-test-1.svg")
            ),
            "graph_measurements_2": lambda: graph_measurements(
                1693405727116,
                1693408802116,
                save_path=str(CHARTS_PATH / "better-chart-1693405727116-1693408802116.svg")
            ),
        }
    )


if __name__ == "__main__":
    main()
