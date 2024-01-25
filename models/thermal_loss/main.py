from pathlib import Path
from pandas import DataFrame
from visualisation import line_chart
from thermal_loss.model_glm import train_model, predict_series
import matplotlib.pyplot as plt

current_dir = Path(__file__).parent.resolve()


# Results:
#   Resample Time        R2       MSE
#             min  0.031018  0.007470
#             10S  0.075911  0.008310
#             30S  0.387229  0.009558
#              1T  0.633589  0.013524
#              5T  0.886545  0.063428 <- This is a pretty good candidate
#             10T  0.894603  0.220654
#             20T  0.929329  0.806990
#             30T  0.927907  0.730623
#              1H  0.841644  4.034842
def train_candidates():
    results = []

    for resample_time in [
        "min",  # This is computed as the largest time difference between any two measurements
        "10S",
        "30S",
        "1T",
        "5T",
        "10T",
        "20T",
        "30T",
        "1H",
    ]:
        results.append(train_model(resample_time))

    return DataFrame(results, columns=["Resample Time", "R2", "MSE", "Model"])


def main():
    [_, r2, mse, model] = train_model("5T")

    df = predict_series(
        model, initial=[92.0, 75.0], time_increment=5 * 60 * 1000, count=90
    )

    fig, _ = line_chart(
        df,
        "5 second future temperature loss prediction",
        x_col="time",
        y_cols=[
            "boiler_temp_c_pred",
            "grouphead_temp_c_pred"
        ],
        ycol_labels={
            "boiler_temp_c_pred": "Boiler Temp (°C)",
            "grouphead_temp_c_pred": "Grouphead Temp (°C)",
        },
    )
    fig.savefig(
        current_dir / "../../docs/diagrams/thermal-loss-model.svg", bbox_inches="tight"
    )