from typing import List
from pandas import DataFrame
import seaborn as sns
import matplotlib.pyplot as plt
import matplotlib.dates as mdates


def graph(groups: List[DataFrame], title: str):
    sns.set(style="whitegrid")

    fig, axes = plt.subplots(nrows=4, ncols=4, figsize=(15, 10))

    fig.suptitle(title)

    for i, ax in enumerate(axes.flat):
        if i < len(groups):
            df = groups[i]

            sns.lineplot(data=df, x="time", y="boiler_temp_c", color="red", ax=ax)
            sns.lineplot(data=df, x="time", y="grouphead_temp_c", color="blue", ax=ax)

            ax.set_title(f"{df['time'].min().strftime('%A, %e %b')}")
            ax.set_xlabel("Time")
            ax.set_ylabel("Temp (°C)")

            date_fmt = mdates.DateFormatter("%H:%M:%S")
            ax.xaxis.set_major_formatter(date_fmt)
            ax.xaxis.set_major_locator(mdates.AutoDateLocator())

            ax.tick_params(axis="x", rotation=45)

    plt.tight_layout()

    plt.show()


def graph_predicted(groups: List[DataFrame], title: str):
    sns.set(style="whitegrid")

    fig, axes = plt.subplots(nrows=2, ncols=4, figsize=(15, 10))

    fig.suptitle(title)

    for i, ax in enumerate(axes.flat):
        if i < len(groups):
            df = groups[i]

            sns.lineplot(data=df, x="time", y="boiler_temp_c_change_actual", color="blue", ax=ax)
            sns.lineplot(data=df, x="time", y="boiler_temp_c_pred", color="red", ax=ax)

            ax.set_title(f"{df['time'].min().strftime('%A, %e %b')}")
            ax.set_xlabel("Time")
            ax.set_ylabel("Temp (°C)")

            date_fmt = mdates.DateFormatter("%H:%M:%S")
            ax.xaxis.set_major_formatter(date_fmt)
            ax.xaxis.set_major_locator(mdates.AutoDateLocator())

            ax.tick_params(axis="x", rotation=45)

    plt.tight_layout()

    plt.show()

def compare_plot(df: DataFrame, title: str, x_col: str, y_cols: List[str]):
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
    plt.show()
