= Hardware

#include "1-hardware-silvia.typ"

== Modifications

=== Power Relay

Before software can be written I need a platform that will provide _real-time temperature data from the boiler and the grouphead_, a mechanism to _control the heating element_, and an _optional thermofilter temperature sensor_.

Manual control over the boiler is important to our application for these reasons:

1. If the machine is always assumed to be on any state recordings we make could be invalidated. For example if we are measuring how much the temperature increases with various heat levels but the machine automatically powered off, the measurements would be invalidated because it is not possible to know when the power was shut off.
2. A feature of the software will be to programmatically run experiments like the heat measurement example above. Having no direct control of power status makes this impossible.

Due to the auto-off mechanism we cannot use an external power switch solution, we will need to modify the silvia's power system.

To solve this problem I use the Shelly 1 @shellyrelaymanual #sym.dash.em a small and cheap ESP8266-based WiFi relay switch. The ESP8266 is a chipset that is supported by the ESPHome project, which is a customisable firmware for integrating ESP chipset devices with IoT infrastructure.

The configuration and instructions for flashing the Shelly switch can be found in my source code under `config/relay_switch`.

#figure(
    image("../diagrams/silvia-shelly-electrical-diagram.svg", width: 80%),
    caption: [Silvia #sym.amp Shelly 1 Electrical Layout],
) <shelly-relay-switch>

The modified layout:

- Removes the cable going to CPU pin 3, which is used to sense whether the machine is being used and delay the auto off function
- Disconnects the power LED (denoted *LS*) #sym.dash.em a 3.3V DC LED will be wired to the Raspberry Pi instead.

The relay will connect to an MQTT broker that will enable the Gesha software to read and control the power status.

=== *T1* thermocouple replacement

The *T1* thermocouple controls the temperature of the boiler during normal operation. It is a temperature-actuated switch that is afixed to the boiler in order to sense the temperature. When the switch is actuated the circuit to the heating element is complete and water will begin heating.

I replace the thermocouple and connect its inputs to Fotek SSR-40 DA Solid-State Relay (*SSR*).

The datasheet @fotek-ssr-40-datasheet states that the SSR responds within 8.3ms and is zero-crossing, meaning that the power change will only be made when the AC power waveform goes to 0.

Next, I place a type K thermocouple where *T1* used to be on the boiler. This is attached to a MAX31855 @max31855-datasheet thermocouple amplifier, which is wired to the Pi's SPI0.0 interface.

Other thermocouples such as a Resistance Temperature Detector (RTD) with a PT100 sensor are also possible. The Type K + MAX31855 combination was chosen due to its low cost and ease of integration.

#figure(
  grid(
    columns: 2,
    gutter: 16pt,
    image("../photos/IMG_2483.jpeg"),
    image("../photos/IMG_2486.jpeg")
  ),
  caption: [Thermocouple against a non-conductive thermal pad (left), and secured in place (right)],
) <ssr-thermocouple>

The Type K thermocouple is an inexpensive thermocouple that can sense temperatures between -200 #sym.degree.c to +1350 #sym.degree.c. Its temperature response curve is nonlinear, however, with more temperature variability in lower temperatures.

Two wires, one made from Chromel and the other from Alumel are joined at one end and open on the other. The MAX31855 uses the different resistance between the two metals to determine the temperature at the junction using the Seebeck effect.

Because this works by passing a current through the wire, the thermocouple probe cannot touch other electrically conductive materials, and must be insulated with a silicone pad.

=== Grouphead thermocouple

#columns(2)[
The grouphead temperature will be used to determine the degree to which the machine has been preheated. Placing a probe on the grouphead itself is ideal because it is the primary component that the boiler water will lose heat to.

The espresso machine front cover is disassembled, exposing the group head metal block. I place a non-conductive thermal pad on the metal and then place the thermocouple on top of that. I secure the thermocouple in place with a high temperature polyimide film adhesive.

#colbreak()

#figure(
  image("../photos/IMG_2392.jpeg"),
  caption: [Type K thermocouple attached to the grouphead],
) <grouphead-thermocouple>

The Type K thermocouple is attached to a second MAX31855 adaptor, attached to the Pi on the SPI0.1 interface.
]

#include "1-hardware-thermofilter.typ"

== Raspberry Pi

I use a Raspberry Pi Zero as the primary platform for integrating the temperature sensors and relays. The Pi Zero is a single-board-computer that is small enough to be mounted inside of the Silvia. It includes a General-purpose Input/Output interface (GPIO) that I will use to interface with the MAX31855 modules (SPI) and the SSR (BCM).

The Pi is provided with power via an external USB power supply and the USB cable is routed into the espresso machine. Additionally I route an ethernet cable, which is attached to a micro-USB 100BASE-T ethernet adaptor. The version of the Pi I use does support WiFi, however due to the low-latency nature of the sensor readings I have chosen a wired network.

#figure(
  image("../diagrams/pi-zero-pinout.svg", width: 80%),
  caption: [Pi GPIO, MAX31855, #sym.amp SSR wiring diagram],
) <pi-zero-pinout>

The Pi Zero's SPI0 interface has 2 *Chip Selects*, meaning at most two MAX31855s to be wired to that interface. The third MAX31855 can be used to connect the thermofilter, and is wired to the SPI1 interface, but this will not be used as mentioned in the Thermofilter section #sym.dash.em I will leave support in place in case it is needed. The SSR is connected to GPIO 26.

#figure(
  image("../diagrams/silvia-shelly-pi-electrical-diagram.svg", width: 80%),
  caption: [Complete wiring diagram including all modifications],
) <silvia-shelly-pi>

This concludes the hardware modifications that have been made to the machine. I now have a hardware platform that will enable me to read the *boiler*, *grouphead*, and *thermofilter* temperatures, as well as to toggle the heating element.
