from pathlib import Path
from numpy import float64

from pandas import read_csv, to_datetime
import matplotlib.pyplot as plt
import seaborn as sns

from visualisation import line_chart


def main():
    current_dir = Path(__file__).parent.resolve()
    measurements = read_csv(
        current_dir / "measurements.csv",
        dtype={
            "boiler_temp_c": float64,
            "grouphead_temp_c": float64,
        },
    )
    measurements["time"] = to_datetime(measurements["time"], unit="ms")
    measurements = measurements.set_index("time").resample("500L").mean().reset_index()

    (_, ax) = line_chart(
        measurements,
        "",
        "time",
        ["boiler_temp_c"],
    )

    ax.axhline(y=100, linestyle="--", color="red")
    ax.set_xlabel("Time")
    ax.set_ylabel("Boiler temperature (°C)")
    ax.set_ylim(80, 120)

    is_heat_on = False
    start_time = None

    for t, heat_level in zip(measurements["time"], measurements["heat_level"]):
        if heat_level == 1.0 and not is_heat_on:
            start_time = t
            is_heat_on = True
        elif heat_level == 0.0 and is_heat_on:
            ax.axvspan(start_time, t, color="red", alpha=0.1, zorder=0)
            on_time = (t - start_time).total_seconds()
            if on_time > 2:
                ax.annotate(
                    f"{(t - start_time).total_seconds()}s",
                    (start_time, 115),
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

    ax.legend().set_visible(False)

    plt.savefig(current_dir / "boiler_temp_sawtooth.svg", bbox_inches="tight")
