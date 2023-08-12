# The raw measurements may be disjoint in time,
# we need to sort them into time-contiguous buckets of measurements.
#
# The measurements are considered contiguous if they occur within 3 seconds of one another.
# Records are normally recorded at most 1 second apart, so a 3x error margin should be appropriate.

from typing import List
from numpy import concatenate, diff, hstack
from pandas import DataFrame
from datetime import timedelta


def group_measurements(measurements: DataFrame) -> List[DataFrame]:
    measurement_groups = measurements.groupby(
        (measurements["time"].diff() > timedelta(seconds=10)).cumsum()
    )

    groups = []

    for _, group in measurement_groups:
        # The total temperature swing should be more than 50 degrees
        # very conservative drop, assuming starting at 80 deg and dropping to a room temperature of 30 deg)
        temperature_condition = group["boiler_temp_c"].max() - group[
            "boiler_temp_c"
        ].min() > (80 - 30)

        # The measurements take place over a time period of at least 4 hours
        time_period_condition = group["time"].max() - group["time"].min() > timedelta(
            hours=4
        )

        if temperature_condition and time_period_condition:
            # Crop the dataframe to only include datapoints that slope downward.
            # Without this, some dataframes include a peak and then a steady drop.
            peak = max(
                [group["boiler_temp_c"].idxmax(), group["grouphead_temp_c"].idxmax()]
            )

            groups.append(group.loc[peak:])

    return groups


def resample_data(data: DataFrame, resample_time: str) -> DataFrame:
    df = data.set_index("time").resample(resample_time).last()

    for is_nan in df.isna().any():
        if is_nan:
            raise ValueError(f"After resampling there were NaN values found")

    df.reset_index(inplace=True)

    return df


def get_largest_timediff(data: DataFrame):
    return data["time"].diff().max()


# Prepares a list of training data into X and y
def prepare(data_frames: List[DataFrame]):
    X = None
    y = None

    for df in data_frames:
        boiler_temps = df["boiler_temp_c"].values
        grouphead_temps = df["grouphead_temp_c"].values

        if X is None:
            X = hstack(
                (boiler_temps[:-1].reshape(-1, 1), grouphead_temps[:-1].reshape(-1, 1))
            )
            y = diff(boiler_temps).reshape(-1, 1)

        else:
            X = concatenate(
                [
                    X,
                    hstack(
                        (
                            boiler_temps[:-1].reshape(-1, 1),
                            grouphead_temps[:-1].reshape(-1, 1),
                        )
                    ),
                ]
            )
            y = concatenate([y, diff(boiler_temps).reshape(-1, 1)])

    return X, y


# Prepares a list of training data into X and y
def split_x_y(data_frames: List[DataFrame], X_col: List[str], y_col: str):
    X = None
    y = None

    for df in data_frames:
        Xs = hstack([df[x_col].values[:-1].reshape(-1, 1) for x_col in X_col])
        ys = diff(df[y_col].values).reshape(-1, 1)

        if X is None:
            X = Xs
            y = ys

        else:
            X = concatenate(
                [
                    X,
                    Xs,
                ]
            )
            y = concatenate([y, ys])

    return X, y