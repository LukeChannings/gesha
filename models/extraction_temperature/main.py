from matplotlib import pyplot as plt
import numpy as np

from pandas import DataFrame, concat
import tensorflow as tf
from extraction_temperature.dataset import (
    get_extraction_temperature_data,
    summary,
)
from extraction_temperature.regression import train_linear_regression
from extraction_temperature.nn import test_model, train_lstm_neural_network, train_neural_network
from util import CHARTS_PATH, MODELS_PATH, run_cli
from visualisation import format_ms, group_chart, line_chart
from seaborn import heatmap

dataset = get_extraction_temperature_data()


def train_linear_regression_model():
    df = concat(dataset)

    X = df[["grouphead_temp_c", "boiler_temp_c"]]
    y = df["thermofilter_temp_c"]

    model = train_linear_regression(X, y)

    return model


def plot_shots():
    group_chart(
        dataset,
        ncols=5,
        nrows=len(dataset) // 5,
        figsize=(15, 15),
        subplot=lambda df, ax: line_chart(
            df,
            ax,
            x_col="time",
            y_cols=["thermofilter_temp_c", "grouphead_temp_c", "boiler_temp_c"],
            ycol_labels={
                "thermofilter_temp_c": "Thermofilter",
                "grouphead_temp_c": "Grouphead",
                "boiler_temp_c": "Boiler",
            },
            hide_legend=True,
            x_tick_formatter=format_ms,
        ),
    )

    plt.savefig(
        CHARTS_PATH / "shots-with-extraction-temperature.svg",
        transparent=True,
        bbox_inches="tight",
    )


def plot_shots_with_prediction():
    model = tf.keras.models.load_model(
        str(MODELS_PATH / "extraction_temperature/output/extraction_temperature_lstm.onnx.tf")
    )

    def add_prediction(df: DataFrame):
        df["thermofilter_temp_c_pred"] = np.zeros(len(df))

        X = df.iloc[:-(len(df) % 20)][["boiler_temp_c", "grouphead_temp_c"]].to_numpy().astype(np.float32).reshape(-1, 10, 2)
        y = np.array(model.predict(X)).reshape(-1) if len(X) > 0 else []

        l = len(df) // len(y) if len(y) > 0 else None
        if l != None:
            for i, _y in enumerate(y):
                df["thermofilter_temp_c_pred"][(i * l):min(len(df), len(df) if i == len(y) - 1 else (i+1) * l)] = _y

        return df

    shots_with_sufficient_measurements = [df for df in dataset if len(df) > 30]

    dataset_with_predictions = [add_prediction(df) for df in shots_with_sufficient_measurements[0:4]]

    group_chart(
        dataset_with_predictions,
        title="Shots",
        subplot=lambda df, ax: line_chart(
            df,
            ax,
            x_col="time",
            y_cols=["thermofilter_temp_c", "thermofilter_temp_c_pred", "grouphead_temp_c", "boiler_temp_c"],
            ycol_labels={
                "thermofilter_temp_c": "Thermofilter",
                "thermofilter_temp_c_pred": "Thermofilter (pred)",
                "grouphead_temp_c": "Grouphead",
                "boiler_temp_c": "Boiler",
            },
            hide_legend=True,
            x_tick_formatter=format_ms,
        ),
    )

    plt.savefig(
        CHARTS_PATH / "shots-with-extraction-temperature-predictions.svg",
        transparent=True,
        bbox_inches="tight",
    )

def plot_correlation():
    df = concat(dataset)[
        ["grouphead_temp_c", "boiler_temp_c", "thermofilter_temp_c"]
    ].corr()
    # Plot the heatmap

    plt.figure(figsize=(10, 7))

    labels = ["Grouphead temperature", "Boiler temperature", "Thermofilter temperature"]

    heatmap(
        df,
        xticklabels=labels,
        yticklabels=labels,
        annot=True,
        cmap="RdBu",
        vmin=-1,
        vmax=1,
    )

    plt.savefig(
        CHARTS_PATH / "extraction-temperature-correlation.svg",
        bbox_inches="tight",
    )


def main():
    run_cli(
        {
            "summary": lambda: summary(dataset),
            "train_linear_regression_model": train_linear_regression_model,
            "train_neural_network_model": lambda: train_neural_network(concat(dataset)),
            "train_lstm_neural_network": lambda: train_lstm_neural_network(concat(dataset)),
            "plot_shots": plot_shots,
            "plot_shots_with_prediction": plot_shots_with_prediction,
            "plot_correlation": plot_correlation,
            "test_model": lambda: test_model(
                concat(dataset),
                str(
                    MODELS_PATH
                    / "extraction_temperature/output/extraction_temperature.onnx"
                ),
            ),
        }
    )


if __name__ == "__main__":
    main()
