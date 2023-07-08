# Experiments

This is an index of all data-gathering experiments, with datasets being CSV, and extracted using the Gesha software.

Terms:

- "Thermocouples" refer to the three thermocouples + MAX31855 amplifier combination, where the three are:
    1. boiler temperature
    2. group head temperature
    3. thermofilter (temperature in the brew basket)

The temperature read / response loop is set to *8 milliseconds* unless otherwise noted.

## `a`-`d`

**THERMOCOUPLE CALIBRATION**

These experiments were designed to calibrate the thermocouples against an [ANOVA sous vide](https://anovaculinary.com/products/anova-precision-cooker), the idea being that the temperature of the water can be increased from freezing to near-boiling in a controlled manner.

Due to the thermocouples being ungrounded, the interference created by the sous vide created sporadic ground faults when reading the temperatures.

## `e`

**THERMOCOUPLE CALIBRATION**

The thermocouples were all placed against the boiler, and the boiler was heated up and then allowed to cool down naturally.

These data will allow me to calibrate the thermocouples against each other, so they all report the same temperatures as each other. The raw data shows that the thermocouples vary by up to 2 &deg;C from one another.

## `f`

**THERMOCOUPLE CALIBRATION**

I placed each thermocouple in a pot of water, which I then brought to a rolling boil. The theory is that the water cannot exceed 100 &deg; C (at sea level), and this may provide a good calibration point for the thermocouples.

## `g`

**TEMPERATURE DELTA MODELLING**

The machine was heated with the thermocouples afixed to the machine in the correct locations.

There was insufficient water flow through the thermofilter, making the data useless.


## `h`

**TEMPERATURE DELTA MODELLING**

- Cold start.
- Brought to a target of 100 &deg; C using the threshold controller.
- Shot pulled against the thermofilter marking the time with the [`time`](../src/bin/time.rs) command

## `i`

This experiment gathers data on the temperature response curve when using a threshold controller set to 97 &deg; C from a cold start, with no brewing.
