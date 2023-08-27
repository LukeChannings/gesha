"""Gesha

Usage:
  gesha get-temperature <sensor>
  gesha set-mode <mode>
  gesha set-control-method <control-method>
  gesha set-target-temperature <temperature>
  gesha set-boiler-level <level>
  gesha get-history <from> <to> [--limit=<limit>] [--bucket-size=<bucket-size>] [--format=<format>] [--out-csv=<csv-file>]
  gesha (-h | --help)

Options:
  -h --help     Show this screen.
"""

import csv
import json
from docopt import docopt

from gesha_api import Gesha

if __name__ == "__main__":
    gesha = Gesha()
    arguments = docopt(__doc__, version="1.0")

    if arguments["get-temperature"]:
        temp = gesha.get_latest_temp(arguments["<sensor>"])
        print(temp)

    if arguments["get-history"]:
        from_ = int(arguments.get("<from>"))
        to = int(arguments.get("<to>"))

        limit = arguments.get("--limit")
        bucketSize = arguments.get("--bucket-size")

        history = gesha.get_measurement_history(
            from_,
            to,
            limit=int(limit) if limit != None else None,
            bucket_size=int(bucketSize) if bucketSize != None else None,
        )

        measurements = history.result()

        output_csv_filename = arguments.get("--out-csv")

        if output_csv_filename:
            with open(output_csv_filename, mode="w") as history_file:
                fieldnames = ["time", "boilerTempC", "groupheadTempC", "heatLevel"]
                writer = csv.DictWriter(
                    history_file,
                    delimiter=",",
                    quotechar='"',
                    quoting=csv.QUOTE_MINIMAL,
                    extrasaction="ignore",
                    fieldnames=fieldnames,
                )

                writer.writeheader()
                writer.writerows(measurements)

                print(f"Wrote {output_csv_filename} with {len(measurements)} records")
        else:
            print(json.dumps(measurements, indent=2))
