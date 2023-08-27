from pathlib import Path
import sys
from numpy import bool_, float64, int64
from pandas import read_csv, to_datetime
import matplotlib.pyplot as plt

from visualisation import group_line_chart

current_dir = Path(__file__).parent.resolve()


# The raw thermofilter.csv file contains measurements from the thermofilter every 100ms for a larger period of time
# than we're interested in. This function filters the measurements down to only the timer periods where we were pulling shots
# to create a smaller file that we can work with a bit more easily.
def filter_thermofilter_measurements():
    thermofilter = read_csv(
        current_dir / "thermofilter.csv",
        dtype={
            "thermofilter_temp_c": float64,
        },
    )

    thermofilter["time"] = to_datetime(thermofilter["time"], unit="ms")

    shots = read_csv(
        current_dir / "shots.csv",
        dtype={
            "total_time": int64,
            "brew_temp_average_c": float64,
            "grouphead_temp_avg_c": float64,
        },
    )

    shots["start_time"] = to_datetime(shots["start_time"], unit="ms")
    shots["end_time"] = to_datetime(shots["end_time"], unit="ms")

    filtered_thermofilter = thermofilter.loc[
        thermofilter["time"].apply(
            lambda time: any(
                [
                    shot["start_time"] <= time <= shot["end_time"]
                    for _, shot in shots.iterrows()
                ]
            )
        )
    ]

    filtered_thermofilter.to_csv(
        current_dir / "thermofilter_filtered.csv",
        index=False,
    )


# The thermofilter and boiler temperature measurements were taken seperately,
# so we need to combine them into a single dataframe so that we can plot them together.
def read_extraction_temp_measurements():
    measurements = read_csv(current_dir / "measurements.csv")
    measurements["time"] = to_datetime(measurements["time"], unit="ms")
    measurements["boiler_temp_c"] = measurements["boiler_temp_c"].astype(float64)
    measurements["grouphead_temp_c"] = measurements["grouphead_temp_c"].astype(float64)
    measurements["start_time"] = measurements["start_time"].astype(int64)
    measurements["end_time"] = measurements["end_time"].astype(int64)

    measurements.set_index("time", inplace=True)

    measurements_thermofilter = read_csv(current_dir / "thermofilter.csv")
    measurements_thermofilter["time"] = to_datetime(measurements_thermofilter["time"], unit="ms")

    groups = []

    for _, group in measurements.groupby("start_time"):
        group_thermofilter_measurements = measurements_thermofilter.loc[
            measurements_thermofilter["time"].apply(
                lambda time: group.index[0] <= time <= group.index[-1]
            )
        ]
        group_thermofilter_measurements = group_thermofilter_measurements.set_index("time").resample("100L").last()
        local_measurements = group.resample("100L").last()

        # Skip shots where we don't have thermofilter measruements (which is most of them I reckon)
        if len(group_thermofilter_measurements) == 0:
            continue

        measurments_combined = local_measurements.join(group_thermofilter_measurements, how="inner").fillna(axis="index", method="ffill").reset_index()

        groups.append(measurments_combined)

    return groups

# This makes a line chart comparing the boiler, grouphead and thermofilter temperatures for every shot pulled.
def make_chart():
    groups = read_extraction_temp_measurements()

    group_line_chart(
        groups,
        "",
        x_col="time",
        y_cols=["boiler_temp_c", "grouphead_temp_c", "thermofilter_temp_c"],
        subplot_title_fn=lambda df: f"{df['boiler_temp_c'].max() - df['thermofilter_temp_c'].max()}\u00b0C ∆",
        date_format="%S",
    )

    plt.show()

def convert_filtered():
    filtered = read_csv(current_dir / "thermofilter_filtered.csv", parse_dates=["time"])
    filtered["time"] = filtered["time"].astype(int64) // 10 ** 6
    filtered.to_csv(current_dir / "thermofilter_filtered_2.csv", index=False)

def main():
    commands = {
        "filter_measurements": filter_thermofilter_measurements,
        "make_chart": make_chart,
        "convert_filtered": convert_filtered,
    }

    args = sys.argv[1:]

    if len(args) == 0 or args[0] not in commands:
        if len(args) > 0:
            print(f"Unknown command: {args[0]}")
        else:
            print("No command specified")

        print(f"Available commands: {', '.join(commands.keys())}")
        return

    for cmd, fn in commands.items():
        if cmd == args[0]:
            fn()
            return

