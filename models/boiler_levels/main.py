from pathlib import Path
from matplotlib import pyplot as plt
import pandas as pd
from pandas import DataFrame, read_csv, to_datetime, to_timedelta
from visualisation import line_chart

current_dir = Path(__file__).parent.resolve()

def get_heat_df(basename: str) -> (DataFrame, float):
    combined_df = DataFrame()
    grouphead_temps = None

    max_len = 0

    for heat_level in range(1, 11):
        df = read_csv(
            current_dir / f"{basename}-{heat_level}.csv"
        )

        df.columns = [snake_case_to_camel_case(col) for col in df.columns]

        df["time"] = to_datetime(df["time"], unit="ms")
        df = df.set_index("time").resample("1S").median().reset_index()

        combined_df[f"heatLevel{heat_level}"] = df["boilerTempC"]
        grouphead_temps = df["groupheadTempC"] if grouphead_temps is None else pd.concat([grouphead_temps, df["groupheadTempC"]])

        max_len = max(max_len, len(df))

    print(combined_df)

    return combined_df, grouphead_temps


def main():
    df_15s, grouphead_temp_mean = get_heat_df("measurement-history-heat-level-15s")

    # fig, _ = line_chart(
    #     df_15s,
    #     f"Heat applied for 15s - Grouphead temp: {grouphead_temp_mean:.2f} C",
    #     "time",
    #     [f"heatLevel{i}" for i in range(1, 11)],
    #     ycol_labels={f"heatLevel{i}": f"{i / 10}" for i in range(1, 11)},
    # )

    # fig.savefig(current_dir / "../../docs/diagrams/heat-level-15s.svg", bbox_inches="tight")

    # df_30s = get_heat_df("measurement-history-heat-level")

    # fig, _ = line_chart(
    #     df_30s,
    #     f"Heat applied for 30s - Grouphead temp: {df_30s['groupheadTempC'].mean():.2f} C",
    #     "time",
    #     [f"heatLevel{i}" for i in range(1, 11)],
    #     ycol_labels={f"heatLevel{i}": f"{i / 10}" for i in range(1, 11)},
    #     x_label="Time (s)",
    #     y_label="Temperature increase (°C)",
    # )

    # fig.savefig(current_dir / "../../docs/diagrams/heat-level-30s.svg", bbox_inches="tight")

def snake_case_to_camel_case(snake_str):
    components = snake_str.split('_')
    camel_case_str = components[0] + ''.join(x.title() for x in components[1:])
    return camel_case_str
