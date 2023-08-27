import math
from typing import Callable, List
from matplotlib.axes import Axes
from matplotlib.figure import Figure
from pandas import DataFrame
import seaborn as sns
import matplotlib.pyplot as plt
import matplotlib.dates as mdates


def group_line_chart(
    groups: List[DataFrame],
    title: str,
    x_col: str,
    y_cols: List[str],
    subplot_title_fn: Callable[[DataFrame], str] = lambda _: f"Title",
    date_format: str = "%H:%M:%S",
) -> plt:
    sns.set(style="whitegrid")

    fig, axes = plt.subplots(
        nrows=math.floor(math.sqrt(len(groups))),
        ncols=math.ceil(math.sqrt(len(groups))),
        figsize=(15, 10),
    )

    fig.suptitle(title)

    for i, ax in enumerate(axes.flat):
        if i < len(groups):
            df = groups[i]

            for y_col in y_cols:
                sns.lineplot(x=x_col, y=y_col, data=df, label=y_col, ax=ax)

            ax.set_title(subplot_title_fn(df))
            ax.set_xlabel("Time")
            ax.set_ylabel("Temp (°C)")

            date_fmt = mdates.DateFormatter(date_format)
            ax.xaxis.set_major_formatter(date_fmt)
            ax.xaxis.set_major_locator(mdates.AutoDateLocator())
            ax.legend().set_visible(False)

            ax.tick_params(axis="x", rotation=45)

    fig.tight_layout()

    return fig

def line_chart(df: DataFrame, title: str, x_col: str, y_cols: List[str]) -> (Figure, Axes):
    fig, ax = plt.subplots(figsize=(10, 6))

    sns.set(style="whitegrid")

    for y_col in y_cols:
        sns.lineplot(x=x_col, y=y_col, data=df, label=y_col, ax=ax)

    ax.set_title(title)
    ax.set_xlabel(x_col)

    date_fmt = mdates.DateFormatter("%H:%M:%S")
    ax.xaxis.set_major_formatter(date_fmt)
    ax.xaxis.set_major_locator(mdates.AutoDateLocator())

    ax.legend()

    return fig, ax
