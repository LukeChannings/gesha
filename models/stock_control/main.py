from pandas import to_datetime
import matplotlib.pyplot as plt
from util import CHARTS_PATH, get_measurements
from visualisation import line_chart


def main():
    measurements = get_measurements(
        to_datetime(1692713400000, unit="ms"),
        to_datetime(1692716550000, unit="ms"),
    )

    measurements = (
        measurements.set_index("time")
        .resample("5S")
        .mean()
        .reset_index()
        .dropna(axis="index")
    )

    _, ax = plt.subplots(figsize=(10, 5))

    line_chart(
        measurements,
        ax,
        title=f"Rancilio Silvia boiler temperature",
        x_col="time",
        y_cols=["boiler_temp_c"],
        x_label="Time",
        y_label="Boiler temperature (°C)",
        hide_legend=True,
        hide_x_ticks=True,
    )

    ax.axhline(y=100, linestyle="--", linewidth=1, color="red")

    is_heat_on = False
    start_time = None

    for t, heat_level in zip(measurements["time"], measurements["heat_level"]):
        if heat_level == 1.0 and not is_heat_on:
            start_time = t
            is_heat_on = True
        elif heat_level == 0.0 and is_heat_on:
            is_heat_on = False
            on_time = (t - start_time).total_seconds()
            ax.axvspan(start_time, t, color="red", alpha=0.1, zorder=0)
            if on_time > 2:
                ax.annotate(
                    f"{on_time}s",
                    (start_time, 97.0),
                    color="black",
                    fontsize=8,
                    fontweight="bold",
                )

            is_heat_on = False

    # If the last data point is also above 100°C
    if is_heat_on:
        ax.axvspan(
            start_time, measurements["time"].iloc[-1], color="red", alpha=0.1, zorder=0
        )  # Set zorder to 0

    plt.savefig(CHARTS_PATH / "boiler_temp_sawtooth.svg", bbox_inches="tight")
