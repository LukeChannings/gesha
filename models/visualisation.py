from datetime import timedelta
import math
from typing import Callable, Dict, List, Optional, Tuple
from matplotlib import dates, ticker
from matplotlib.axes import Axes
from numpy import around, linspace
from pandas import DataFrame, to_datetime
import seaborn as sns
import matplotlib.pyplot as plt

from util import CHARTS_PATH, get_measurements


# Renders charts in a grid, composable via a subplot function
def group_chart(
    groups: List[DataFrame],
    title: str = None,
    figsize: Tuple[int, int] = (15, 10),
    subplot: Callable[[DataFrame, Axes], None] = lambda df, ax: None,
    nrows = None,
    ncols = None,
):
    fig, axes = plt.subplots(
        nrows=math.floor(math.sqrt(len(groups))) if nrows is None else nrows,
        ncols=math.ceil(math.sqrt(len(groups))) if ncols is None else ncols,
        figsize=figsize,
    )

    for i, ax in enumerate(axes.flat):
        if i >= len(groups):
            fig.delaxes(ax)
            continue

        df = groups[i]
        subplot(df, ax)

    # Get the legend handles and labels from one of the subplots
    handles, labels = axes[0, 0].get_legend_handles_labels()

    # Create a legend for the entire figure using the handles and labels
    fig.legend(handles, labels, loc="lower right", bbox_to_anchor=(1, 1)).set_visible(
        True
    )

    if title:
        fig.suptitle(title)

    fig.tight_layout()

    return fig, ax


def line_chart(
    df: DataFrame,
    ax: Axes,
    x_col: str,
    y_cols: List[str],
    x_label: str = "",
    y_label: str = "",
    ycol_labels: Optional[Dict[str, str]] = {},
    title: str = "",
    hide_legend: bool = False,
    title_fontdict: Optional[Dict[str, str]] = {"fontsize": 10},
    label_fontdict: Optional[Dict[str, str]] = {"fontsize": 8},
    x_tick_formatter: Optional[Callable[[float, int], str]] = None,
    hide_x_ticks: bool = False,
):
    for y_col in y_cols:
        if y_col == "target_temp_c":
            sns.lineplot(x=x_col, y=y_col, data=df, label=ycol_labels.get(y_col, y_col), ax=ax, color="red", linestyle="--")
        else:
            sns.lineplot(x=x_col, y=y_col, data=df, label=ycol_labels.get(y_col, y_col), ax=ax)


    ax.set_title(title, fontdict=title_fontdict)
    ax.set_xlabel(x_label, fontdict=label_fontdict)
    ax.set_ylabel(y_label, fontdict=label_fontdict)

    y_min = math.floor(df[y_cols].values.min() / 5) * 5
    y_max = math.ceil(df[y_cols].values.max() / 5) * 5

    ax.set_yticks(
        around(linspace(y_min, y_max, 5) * 4) / 4
    )

    ax.set_yticklabels(ax.get_yticks(), fontdict=label_fontdict)

    if hide_x_ticks:
        ax.set_xticks([])

    if x_tick_formatter is not None:
        ax.xaxis.set_major_formatter(x_tick_formatter)

    if label_fontdict:
        ax.tick_params(axis='x', labelsize=label_fontdict.get("fontsize"))

    if hide_legend:
        ax.legend().set_visible(False)

def distribution_chart(df: DataFrame, x: str):
    sns.histplot(df[x], bins=1000, kde=False)
    plt.xlabel(x)
    plt.ylabel("Frequency")
    plt.title(f"Histogram of {x}")
    plt.show()


# A chart to show a period of measurements
def graph_measurements(from_time: int, to_time: int, title: str = None, save_path = None):
    measurements = get_measurements(
        to_datetime(from_time, unit="ms"),
        to_datetime(to_time, unit="ms"),
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
        title=title,
        x_col="time",
        y_cols=["boiler_temp_c", "grouphead_temp_c", "target_temp_c"],
        ycol_labels={ "boiler_temp_c": "Boiler temperature (C)", "grouphead_temp_c": "Grouphead temperature (C)", "target_temp_c": "Target temperature (C)" },
        x_label="Time",
        y_label="Temperature (°C)",
        x_tick_formatter=dates.DateFormatter("%H:%M")
    )

    ax.set_yticks(
        around(linspace(30, 130, 15) * 4) / 4
    )

    ax.set_yticklabels(ax.get_yticks())

    # ax.axhline(y=100, linestyle="--", linewidth=1, color="red")

    for t, heat_level in zip(measurements["time"], measurements["heat_level"]):
        if heat_level > 0:
            ax.axvspan(t - timedelta(milliseconds=100), t, color="red", alpha=0.1, zorder=0)

    if save_path:
        plt.savefig(save_path, bbox_inches="tight")
    else:
        plt.show()


def format_secs(value, _tick_number):
    m = int((value % 3600) // 60)
    s = int(value % 60)
    return f"{m:02}:{s:02}"


def format_ms(value, tick_number):
    return format_secs(value // 1000, tick_number)
