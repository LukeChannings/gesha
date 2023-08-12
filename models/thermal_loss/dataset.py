# Run ./query.sh to update the data to the latest
from pathlib import Path
from numpy import bool_, float64
from pandas import DataFrame, concat, read_csv, to_datetime


def read_thermal_loss_data() -> DataFrame:
    current_dir = Path(__file__).parent.resolve()
    column_types = {
        "boiler_temp_c": float64,
        "grouphead_temp_c": float64,
        "power": bool_,
    }

    # These measurements are from the local gesha.db dump
    measurements = read_csv(
        current_dir / "measurements.csv",
        dtype=column_types,
    )

    # These are the latest measurements from the server
    measurements_remote = read_csv(
        current_dir / "measurements_remote.csv",
        dtype=column_types,
    )

    measurements = concat([measurements, measurements_remote], axis="index")

    # Ugly, but pandas tells me that using parse_date is deprecated, and date_format won't support UNIX timestamps.
    measurements["time"] = to_datetime(measurements["time"], unit="ms")

    measurements.set_index(measurements["time"], inplace=True)

    return measurements
