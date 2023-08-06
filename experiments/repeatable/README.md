# Repeatable experiments

This is a bunch of python scripts that automate experiments using the MQTT API.

The `gesha_api` module contains a `Gesha` class that can be initialised and used to perform actions.

## Heat Level

Is an experiment designed to run the boiler at every heat level in a controlled manner so that the temperature differences can be measured.

`poetry run python power_level`
