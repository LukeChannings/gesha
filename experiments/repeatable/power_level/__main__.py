from time import sleep, time
from gesha_api import Gesha

HEAT_TIME_SECONDS = 30
HEAT_LAG_WAIT_SECONDS = 60

def main():
    gesha = Gesha()

    # wait a sec for temperature values to come in.
    sleep(1)

    starting_boiler_temperature = gesha.get_latest_temp("boiler")

    # Move into manual control mode.
    gesha.set_control_method("none")
    gesha.set_mode("active")

    print("EXPERIMENT BEGINS")

    for level in (0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0):
        print(f"HEAT ON, {int(time() * 1000)}, {level}")

        a_temp = starting_boiler_temperature = gesha.get_latest_temp("boiler")
        gesha.set_boiler_level(level)

        sleep(HEAT_TIME_SECONDS)

        gesha.set_boiler_level(0)

        print(f"HEAT OFF, {int(time() * 1000)}, {level}")

        # Sometimes there's a large lag before the temperature moves.
        # We'll give it a minute before moving on.
        sleep(HEAT_LAG_WAIT_SECONDS)

        b_temp = gesha.get_latest_temp("boiler")

        print(f"After {HEAT_LAG_WAIT_SECONDS}s the temperature is {b_temp}")
        print(f"The starting temperature for {level} was {a_temp}. Diff: {b_temp - a_temp} degrees C")
        print("Note: this is *not* the maximum. Dump the measurement history for further analysis.")

        # Wait for the temperature to return to the starting temp
        gesha.wait_for_temp_le('boiler', starting_boiler_temperature)

        print(f"TEMPERATURE NORMAL, {int(time() * 1000)}, {level}")

    gesha.set_mode("idle")

if __name__ == "__main__": main()
