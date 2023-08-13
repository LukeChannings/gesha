from typing import List
from pandas import DataFrame
import seaborn as sns
import matplotlib.pyplot as plt
import matplotlib.dates as mdates


def group_line_chart(groups: List[DataFrame], title: str, x_col: str, y_cols: List[str]) -> plt:
    sns.set(style="whitegrid")

    fig, axes = plt.subplots(nrows=4, ncols=4, figsize=(15, 10))

    fig.suptitle(title)

    for i, ax in enumerate(axes.flat):
        if i < len(groups):
            df = groups[i]

            for y_col in y_cols:
                sns.lineplot(x=x_col, y=y_col, data=df, label=y_col, ax=ax)

            ax.set_title(f"{df[x_col].min().strftime('%A, %e %b')}")
            ax.set_xlabel("Time")
            ax.set_ylabel("Temp (°C)")

            date_fmt = mdates.DateFormatter("%H:%M:%S:%f")
            ax.xaxis.set_major_formatter(date_fmt)
            ax.xaxis.set_major_locator(mdates.AutoDateLocator())
            ax.legend().set_visible(False)

            ax.tick_params(axis="x", rotation=45)

    fig.tight_layout()

    return fig

def line_chart(df: DataFrame, title: str, x_col: str, y_cols: List[str]) -> plt:
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
    return fig
