from datetime import datetime, tzinfo
from typing import List, Optional
from numpy import diff, mean
from pandas import DataFrame, to_datetime, concat

from sklearn.linear_model import LinearRegression
from sklearn.metrics import mean_squared_error, r2_score
from sklearn.model_selection import train_test_split

from thermal_loss.dataset import read_thermal_loss_data
from thermal_loss.data_processing import (
    get_resample_time,
    resample,
    group_measurements,
    add_diff_cols,
)


def train_model(resample_time="min"):
    # Get the raw measurements
    thermal_loss_data = read_thermal_loss_data()

    # Group into time-invariant series'
    groups = group_measurements(thermal_loss_data)

    # We're predicting the *rate of change* in degrees from one measurement to the next.
    # X_1 = boiler_temp_c, X_2 = grouphead_temp_c. Y_1 = boiler_temp_c_diff, Y_2 = grouphead_temp_c_diff
    # We need a Multiple Linear Regression because we need to use the predicted result
    # to feed into the next prediction (see `predict_series`)
    groups = [
        add_diff_cols(
            resample(group, get_resample_time(group, resample_time)),
            cols=["boiler_temp_c", "grouphead_temp_c"],
        )
        for group in groups
    ]

    df = concat(groups, axis="index")

    X = df[["boiler_temp_c", "grouphead_temp_c"]].values
    Y = df[["boiler_temp_c_diff", "grouphead_temp_c_diff"]].values

    X_train, X_test, Y_train, Y_test = train_test_split(
        X, Y, test_size=0.33, random_state=42
    )

    model = LinearRegression()
    model.fit(X_train, Y_train)

    Y_pred = model.predict(X_test)

    mse = mean_squared_error(Y_test, Y_pred)
    r2 = r2_score(Y_test, Y_pred)

    return [resample_time, r2, mse, model]


def predict_series(
    model: LinearRegression,
    initial: (float, float),
    time_increment: int,
    count: int,
):
    time = [int(datetime.now().timestamp() * 1000)]
    boiler_temp_c = [initial[0]]
    grouphead_temp_c = [initial[1]]

    for i in range(1, count + 1):
        time.append(time[i - 1] + time_increment)
        boiler_temp = boiler_temp_c[i - 1]
        grouphead_temp = grouphead_temp_c[i - 1]

        [boiler_temp_diff, grouphead_temp_diff] = model.predict(
            [[boiler_temp, grouphead_temp]]
        )[0]

        boiler_temp_c.append(boiler_temp + boiler_temp_diff)
        grouphead_temp_c.append(grouphead_temp + grouphead_temp_diff)

    return DataFrame(
        {
            "time": to_datetime(time, unit="ms"),
            "boiler_temp_c_pred": boiler_temp_c,
            "grouphead_temp_c_pred": grouphead_temp_c,
        }
    )
