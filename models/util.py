from pathlib import Path
import sqlite3
import subprocess
import sys
from typing import Callable, Dict

from pandas import DataFrame, read_sql_query, to_datetime
from string import Formatter

def get_git_root(path="."):
    try:
        root = (
            subprocess.check_output(["git", "rev-parse", "--show-toplevel"], cwd=path)
            .decode("utf-8")
            .strip()
        )
        return root
    except subprocess.CalledProcessError:
        return None

ROOT_PATH = Path(get_git_root())
MODELS_PATH = ROOT_PATH / "models"
CHARTS_PATH = ROOT_PATH / "docs/diagrams"
DATA_PATH = ROOT_PATH / "data"
DB_PATH = DATA_PATH / "gesha-13-09-2023.db"

SQLITE_DB_CONN = sqlite3.connect(DB_PATH)

def pre_heat_level(target_temp, grouphead_temp):
    if target_temp <= 90.0: return grouphead_temp / 74.0
    if target_temp <= 93.0: return grouphead_temp / 76.0
    if target_temp <= 95.0: return grouphead_temp / 78.0
    if target_temp <= 99.0: return grouphead_temp / 80.0
    if target_temp <= 101.0: return grouphead_temp / 82.0
    if target_temp <= 103.0: return grouphead_temp / 84.0
    if target_temp <= 107.0: return grouphead_temp / 86.0
    if target_temp <= 109.0: return grouphead_temp / 88.0
    return grouphead_temp / 90.0


def get_measurements(from_time, to_time):
    query = f"""
    SELECT time, grouphead_temp_c, boiler_temp_c, heat_level, target_temp_c
    FROM measurement
    WHERE time >= {int(from_time.timestamp() * 1000)}
    AND time <= {int(to_time.timestamp() * 1000)}
    """

    df = read_sql_query(query, SQLITE_DB_CONN)
    df["time"] = to_datetime(df["time"], unit="ms")

    return df

def upscale_dataframe(df: DataFrame, period: str):
    return (
        df.set_index("time").resample(period).ffill().reset_index().dropna(axis="index")
    )

def run_cli(commands: Dict[str, Callable]) -> None:
    args = sys.argv[1:]

    if len(args) == 0 or args[0] not in commands:
        if len(args) > 0:
            print(f"Unknown command: {args[0]}")
        else:
            print("No command specified")

        print(f"Available commands: {', '.join(commands.keys())}")
        return

    for cmd, fn in commands.items():
        if cmd == args[0]:
            fn()
            return


def strfdelta(tdelta, fmt='{M:02}m {S:02}s', inputtype='timedelta'):
    """Convert a datetime.timedelta object or a regular number to a custom-
    formatted string, just like the stftime() method does for datetime.datetime
    objects.

    The fmt argument allows custom formatting to be specified.  Fields can
    include seconds, minutes, hours, days, and weeks.  Each field is optional.

    Some examples:
        '{D:02}d {H:02}h {M:02}m {S:02}s' --> '05d 08h 04m 02s' (default)
        '{W}w {D}d {H}:{M:02}:{S:02}'     --> '4w 5d 8:04:02'
        '{D:2}d {H:2}:{M:02}:{S:02}'      --> ' 5d  8:04:02'
        '{H}h {S}s'                       --> '72h 800s'

    The inputtype argument allows tdelta to be a regular number instead of the
    default, which is a datetime.timedelta object.  Valid inputtype strings:
        's', 'seconds',
        'm', 'minutes',
        'h', 'hours',
        'd', 'days',
        'w', 'weeks'
    """

    # Convert tdelta to integer seconds.
    if inputtype == 'timedelta':
        remainder = int(tdelta.total_seconds())
    elif inputtype in ['s', 'seconds']:
        remainder = int(tdelta)
    elif inputtype in ['m', 'minutes']:
        remainder = int(tdelta)*60
    elif inputtype in ['h', 'hours']:
        remainder = int(tdelta)*3600
    elif inputtype in ['d', 'days']:
        remainder = int(tdelta)*86400
    elif inputtype in ['w', 'weeks']:
        remainder = int(tdelta)*604800

    f = Formatter()
    desired_fields = [field_tuple[1] for field_tuple in f.parse(fmt)]
    possible_fields = ('W', 'D', 'H', 'M', 'S')
    constants = {'W': 604800, 'D': 86400, 'H': 3600, 'M': 60, 'S': 1}
    values = {}
    for field in possible_fields:
        if field in desired_fields and field in constants:
            values[field], remainder = divmod(remainder, constants[field])
    return f.format(fmt, **values)
