from datetime import datetime
from typing import List, Optional
from numpy import diff, concatenate, mean
from pandas import DataFrame, to_datetime
from sklearn.linear_model import LinearRegression


def train_model(X: DataFrame, y: DataFrame):
    regressor = LinearRegression()
    regressor.fit(X, y)

    return regressor


def test_model(model: LinearRegression, X):
    return [model.predict([[x[0], x[1]]])[0][0] for x in X]


def predict_series(
    boiler_model: LinearRegression,
    grouphead_model: LinearRegression,
    initial: (float, float),
    time_increment: int,
    count: int,
):
    time = [int(datetime.now().timestamp())]
    boiler_temp_c = [initial[0]]
    grouphead_temp_c = [initial[1]]

    for i in range(1, count + 1):
        time.append(time[i - 1] + time_increment)
        boiler_temp = boiler_temp_c[i - 1]
        grouphead_temp = grouphead_temp_c[i - 1]

        boiler_temp_c.append(boiler_temp + boiler_model.predict( [[boiler_temp, grouphead_temp]] )[0][0])
        grouphead_temp_c.append(grouphead_temp + grouphead_model.predict( [[boiler_temp, grouphead_temp]] )[0][0])

    return DataFrame(
        {
            "time": to_datetime(time, unit="ms"),
            "boiler_temp_c": boiler_temp_c,
            "grouphead_temp_c": grouphead_temp_c,
        }
    )
