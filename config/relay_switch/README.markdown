# Shelly Switch

The Silvia's power state is controlled with a Shelly switch, following the wiring from this [reddit post](https://www.reddit.com/r/espresso/comments/upvbp0/easy_rancilio_silvia_e_to_m_conversion/).

## Wiring

|Silvia E | Shelly 1 |
---|---
1 | Neural
2 | Live
3 | unplugged (cut wire+connector off at lead)
5 | Live
6 | SW
7 | unplugged (Power LED light)
9 | Input
10 | Output

## Shelly ESPHome

The Shelly is flashed with [ESPHome](https://devices.esphome.io/devices/Shelly-1), which is chosen because of its easy integration and configuration in comparison to the default Shelly firmware.
