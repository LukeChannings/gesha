from pathlib import Path
from numpy import bool_, float64
from pandas import read_csv, to_datetime
import matplotlib.pyplot as plt

from visualisation import group_line_chart

def read_extraction_temperature():
    current_dir = Path(__file__).parent.resolve()

    measurements = read_csv(
        current_dir / "measurements.csv",
        dtype={
            "boiler_temp_c": float64,
            "grouphead_temp_c": float64,
            "pull": bool_,
        },
    )

    measurements["time"] = to_datetime(measurements["time"], unit="ms")

    measurements.set_index("time", inplace=True)

    measurements = measurements.resample("250L").last()

    measurements_thermofilter = read_csv(
        current_dir / "measurements_thermofilter.csv",
        dtype={
            'thermofilter_temp_c': float64
        }
    )

    measurements_thermofilter["time"] = to_datetime(measurements_thermofilter["time"], unit="ms")
    measurements_thermofilter.set_index("time", inplace=True)

    measurements_thermofilter = measurements_thermofilter.resample("250L").last()

    df = measurements.join(measurements_thermofilter, how='inner')
    df.fillna(method="ffill", axis="index", inplace=True)
    df["pull"] = bool_(df["pull"])

    return df

def main():
    measurements = read_extraction_temperature()

    # Create a boolean mask to identify contiguous groups where "pull" is true or false
    pull_true_mask = measurements["pull"]
    group_indices = (pull_true_mask != pull_true_mask.shift()).cumsum()

    groups = [group.reset_index() for _, group in measurements.groupby(group_indices) if group["pull"].all() and len(group) > 1]

    group_line_chart(groups, "", x_col="time", y_cols=["boiler_temp_c", "grouphead_temp_c", "thermofilter_temp_c"])
    plt.show()
