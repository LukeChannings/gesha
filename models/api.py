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
from docopt import docopt
from concurrent.futures import Future
import json
from time import sleep, time
from typing import Literal
from os import getenv
import paho.mqtt.client as mqtt
from dotenv import load_dotenv

load_dotenv()

# Set these in the .env file
MQTT_USER = getenv("GESHA_MQTT_USER")
MQTT_PASS = getenv("GESHA_MQTT_PASS")
MQTT_HOST = getenv("GESHA_MQTT_HOST")
MQTT_PORT = int(getenv("GESHA_MQTT_PORT"))

def main():
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

class Gesha:
    client: mqtt.Client

    boiler_temp = []
    grouphead_temp = []
    thermofilter_temp = []

    def __init__(self) -> None:
        self.client = mqtt.Client()

        self.client.username_pw_set(MQTT_USER, MQTT_PASS)
        self.client.connect(MQTT_HOST, MQTT_PORT, 60)

        self.client.subscribe("gesha/#")

        self.client.message_callback_add(
            "gesha/temperature/#", self.handle_temperature_messages
        )
        self.client.loop_start()

    def set_mode(self, mode: Literal["idle", "active", "brew", "steam"]):
        self.client.publish("gesha/mode/set", mode).wait_for_publish()

    def set_control_method(
        self, control_method: Literal["none", "threshold", "pid", "mpc"]
    ):
        self.client.publish(
            "gesha/control_method/set", control_method
        ).wait_for_publish()

    def set_target_temperature(self, temperature: int):
        self.client.publish(
            "gesha/temperature/target/set", temperature
        ).wait_for_publish()

    def set_boiler_level(self, boiler_level: float):
        self.client.publish("gesha/boiler_level/set", boiler_level)

    def get_latest_temp(self, sensor: Literal["boiler", "grouphead", "thermofilter"]):
        match sensor:
            case "boiler":
                self.wait_for_temp("boiler")
                return self.boiler_temp[len(self.boiler_temp) - 1][1]
            case "grouphead":
                self.wait_for_temp("grouphead")
                return self.grouphead_temp[len(self.grouphead_temp) - 1][1]
            case "thermofilter":
                self.wait_for_temp("thermofilter")
                return self.thermofilter_temp[len(self.thermofilter_temp) - 1][1]

    def wait_for_temp(self, sensor: Literal["boiler", "grouphead", "thermofilter"]):
        match sensor:
            case "boiler":
                while len(self.boiler_temp) < 1:
                    sleep(0.1)
            case "grouphead":
                while len(self.grouphead_temp) < 1:
                    sleep(0.1)
            case "thermofilter":
                while len(self.thermofilter_temp) < 1:
                    sleep(0.1)

    def wait_for_temp_le(
        self, sensor: Literal["boiler", "grouphead", "thermofilter"], temp: float
    ):
        while True:
            if self.get_latest_temp(sensor) <= temp:
                break
            else:
                sleep(0.1)

    def wait_for_temp_ge(
        self, sensor: Literal["boiler", "grouphead", "thermofilter"], temp: float
    ):
        while True:
            if self.get_latest_temp(sensor) >= temp:
                break
            else:
                sleep(0.1)

    def get_temps_in_range(
        self,
        sensor: Literal["boiler", "grouphead", "thermofilter"],
        from_: int,
        to: int,
    ):
        temps = []

        match sensor:
            case "boiler":
                temps = self.boiler_temp
            case "grouphead":
                temps = self.grouphead_temp
            case "thermofilter":
                temps = self.thermofilter_temp

        return [(time, temp) for (time, temp) in temps if time < to and time > from_]

    def get_measurement_history(self, from_: int, to: int, limit: int, bucket_size):
        promise = Future()

        def handle_measurements(client, _userdata, message):
            client.message_callback_remove("gesha/temperature/history")
            promise.set_result(json.loads(message.payload))

        self.client.message_callback_add(
            "gesha/temperature/history", handle_measurements
        )

        self.client.publish(
            "gesha/temperature/history/command",
            json.dumps(
                {
                    "from": from_,
                    "to": to,
                    "limit": limit,
                    "bucketSize": bucket_size,
                }
            ),
        )

        return promise

    def handle_temperature_messages(self, _client, _userdata, message):
        match message.topic:
            case "gesha/temperature/boiler":
                self.boiler_temp.append((int(time() * 1000), float(message.payload)))
            case "gesha/temperature/grouphead":
                self.grouphead_temp.append((int(time() * 1000), float(message.payload)))
            case "gesha/temperature/thermofilter":
                self.thermofilter_temp.append(
                    (int(time() * 1000), float(message.payload))
                )

