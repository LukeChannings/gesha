from glob import glob
from typing import List
from pandas import (
    DataFrame,
    Timedelta,
    Timestamp,
    merge,
    read_csv,
    read_json,
    read_sql_query,
    to_datetime,
    concat,
)
from tabulate import tabulate

from util import MODELS_PATH, SQLITE_DB_CONN, get_measurements, upscale_dataframe

cwd_dir = MODELS_PATH / "extraction_temperature"

# Combine all the measurements from the thermofilter into a single dataframe
def get_combined_thermofilter_measurements():
    df = None

    for file in glob((cwd_dir / "aux-silvia/*.csv").as_posix()):
        measurements = read_csv(
            file, header=None, names=["time", "thermofilter_temp_c"]
        )
        df = measurements if df is None else concat([df, measurements], axis="index")

    df["time"] = to_datetime(df["time"], unit="ms")

    for file in glob((cwd_dir / "aux-silvia/*.json").as_posix()):
        measurements = read_json(file)
        measurements.rename(
            columns={"timestamp": "time", "value": "thermofilter_temp_c"}, inplace=True
        )
        df = concat([df, measurements], axis="index")

    df = df.drop_duplicates(subset="time", keep="first", ignore_index=True).sort_values(
        by="time"
    )

    return df


# Query all shots from the database
def get_shots():
    query = """
        SELECT * FROM shot
        -- These shots have been removed because they are outliers
        WHERE start_time != 1692785576858
        AND start_time != 1692785876575
        AND start_time != 1692129457611
        AND start_time != 1692625413417
        AND start_time != 1692721311271
        ORDER BY start_time ASC
    """

    df = read_sql_query(query, SQLITE_DB_CONN)

    df["start_time_unix"] = df["start_time"]
    df["start_time"] = to_datetime(df["start_time"], unit="ms")
    df["end_time"] = to_datetime(df["end_time"], unit="ms")

    return df

# Combine the thermofilter measurements with the shot measurements
def get_extraction_temperature_data() -> List[DataFrame]:
    # Get all the thermofilter measurements upfront
    thermofilter_measurements = get_combined_thermofilter_measurements()

    results = []

    for _, shot in get_shots().iterrows():
        # Get the thermofilter measurements for the shot
        shot_thermofilter_measurements = thermofilter_measurements[
            (thermofilter_measurements["time"] >= shot["start_time"])
            & (thermofilter_measurements["time"] <= shot["end_time"])
        ]

        if len(shot_thermofilter_measurements) == 0:
            continue

        shot_measurements = get_measurements(shot["start_time"], shot["end_time"])

        # Upscale the measurements to 100ms intervals
        shot_thermofilter_measurements = upscale_dataframe(
            shot_thermofilter_measurements, "100L"
        )

        shot_measurements = upscale_dataframe(shot_measurements, "100L")

        # Combine the thermofilter measurements with the rest (boiler, grouphead, etc.)
        combined_measurements = merge(
            shot_measurements, shot_thermofilter_measurements, on="time", how="outer"
        ).dropna(axis="index")

        # Snip off the beginning of the shot where the thermofilter temp is still ~= the grouphead temp
        combined_measurements = combined_measurements.iloc[
            combined_measurements["thermofilter_temp_c"].idxmax() : -10
        ]

        if len(combined_measurements) == 0:
            continue

        # Start the time from 0
        combined_measurements["time"] = (
            combined_measurements["time"] - Timestamp("1970-01-01")
        ) // Timedelta("1ms")
        combined_measurements["time"] = (
            combined_measurements["time"] - combined_measurements["time"].iloc[0]
        )

        combined_measurements["start_time"] = shot["start_time_unix"]

        results.append(combined_measurements)

    return results


def summary(groups):
    table = [
        [
            int(g["start_time"].iloc[0]),
            g["grouphead_temp_c"].max(),
            g["boiler_temp_c"].max(),
            g["thermofilter_temp_c"].max(),
        ]
        for g in groups
    ]

    table.sort(key=lambda row: row[1] + row[2], reverse=True)

    print(f"Total observations: {len(concat(groups))}\n")
    print(concat(groups).describe())
    print(
        tabulate(
            table,
            headers=["Shot ID", "Grouphead Temp", "Boiler Temp", "Extraction Temp"],
        )
    )
