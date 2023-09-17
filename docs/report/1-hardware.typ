= Hardware <hardware>

#include "1-hardware-silvia.typ"

== Raspberry Pi Zero

I use a Raspberry Pi Zero as the primary platform for hosting the software and integrating hardware modifications. The Pi Zero (@pi-zero) is a small single-board-computer (SBC) that I mount inside the casing of the Rancilio Silvia. The device includes a General-purpose Input/Output interface (*GPIO*) that I will use to interface with temperature sensors and relays. It is powered independently of the Silvia, receiving power via a USB cable that is routed underneath the Silvia's chassis. The Pi Zero supports WiFi: however, due to the potential signal problems of being housed inside of a metal case and the response time requirements of the software, I have opted to connect the Pi Zero to the network via an ethernet cable. This is achieved by using a micro-USB 100BASE-T ethernet adaptor, as the Pi Zero does not include an ethernet device onboard.

The Pi Zero is a low performance device, with a 1GHz single-core 32-bit ARM System-on-Chip (SoC), 512MB of RAM, and microSD storage. A faster SBC (the Pi Zero 2) was released in 2021. Due to the electronics supply chain problems and ongoing shortages caused by the COVID-19 pandemic @pennisi2022pandemic, I have been unable to obtain this device for my project. When it is available, I plan to upgrade as the 64-bit architecture is better supported and my application would benefit from multi-threading.

#figure(
    image("../photos/IMG_0373.jpeg", width: 80%),
    caption: [Pi Zero mounted inside the housing of a Rancilio Silvia]
) <pi-zero>

== Modifications to the Rancilio Silvia

Several hardware changes are required to perform this project. In order to build the software platform for controlling the brew session, I need to have hooks into the machine that allow the programmatic retrieval of important features, such as the machine's power-on state, the boiler temperature, etc.

=== Power Relay

I need control over the machine's power state for these reasons:

1. If the machine is always assumed to be powered on, recordings of the machine's state could be difficult or impossible to model. For example, if we are measuring how much the temperature increases with various heat levels but the machine is powered off, the measurements would be invalidated because although our software is turning the boiler on nothing is happening because the machine is powered off.
2. A feature of the software will be to programmatically run experiments like the heat measurement example above. Having no direct control of power status makes this impossible.

It is possible to control the power state of the _M_ model by using smart power plug, but the auto-off mechanism used by the _E_ model makes this impossible, since the power state of the machine is managed by the CPU component and requires manual actuation of the *IS* power switch. To solve this problem I use the Shelly 1 @shellyrelaymanual #sym.dash.em a small and cheap ESP8266-based WiFi relay switch. The ESP8266 is a chipset that is supported by the ESPHome project, which is a customisable firmware for integrating ESP chipset devices with IoT infrastructure.

The configuration and instructions for flashing the Shelly switch can be found in my source code under `config/relay_switch`.

#figure(
    image("../diagrams/silvia-shelly-electrical-diagram.svg", width: 80%),
    caption: [Silvia #sym.amp Shelly 1 Electrical Layout],
) <shelly-relay-switch>

#block(breakable: false, [

The layout is modified as follows (@shelly-relay-switch):

- Removal of the CPU component and replacement by the Shelly 1
- Removal of the cable going to CPU pin 3, which is used to sense whether the machine is being used and delay the auto-off function
- Disconnection of the power LED (denoted *LS*) #sym.dash.em a 3.3V DC LED will be wired to the Raspberry Pi instead.

The relay will connect to an MQTT broker to integrate with the rest of the software, this is described further in the #link(<software-architecture>)[Software Architecture] section.
])

=== Boiler element control

The software is required to control the boiler's heating element, which will enable the implementation of novel control methods. There must be hardware to support this, which requires the modification of the machine's electrical layout.

#block(breakable: false, [

Boiler control requires two things:

1. A power relay that can be integrated with an embedded computer system that can be used to toggle the boiler's heating element, and
2. A temperature sensor to monitor the boiler's temperature

As discussed, the *T1* thermocouple controls the temperature of the boiler during normal operation. It is a temperature-sensitive actuator (@tsa). The underside is affixed to the boiler, and the left and right pins will complete a circuit to the boiler's heating element when the temperature is below the actuation threshold, thus heating the water.
])

#figure(
    image("../photos/IMG_2510.jpeg", width: 40%),
    caption: [A temperature-sensitive actuator from the Rancilio Silvia]
) <tsa>

#block(breakable: false, [
I modify the machine as follows:

- Remove the *T1* TSA
- Place a Type-K thermocouple - insulated by a non-conductive silicone pad - where the *T1* TSA was previously fixed to the boiler.
- Wire the pins previously connected to the *T1* TSA to a Solid-State Relay (SSR)

The SSR I have chosen to use is the Fotek SSR-40 DA. The datasheet @fotek-ssr-40-datasheet states that it responds within 8.3ms and is zero-crossing, meaning that the power change will only be made when the AC power waveform goes to 0.

I attach the Type-K thermocouple to a MAX31855 @max31855-datasheet thermocouple amplifier, which is wired to the Pi Zero's GPIO interface.

Other thermocouples such as a Resistance Temperature Detector (RTD) with a PT100 sensor are also possible. The Type K + MAX31855 combination was chosen due to its low cost and ease of integration. The Type K thermocouple is an inexpensive thermocouple that can sense temperatures between -200 #sym.degree.c to +1350 #sym.degree.c. Its temperature response curve is nonlinear, but only at low (#sym.lt 0 #sym.degree.c) temperatures, so temperature gradients are not a concern in this application.
])

#figure(
  grid(
    columns: 2,
    gutter: 16pt,
    image("../photos/IMG_2483.jpeg"),
    image("../photos/IMG_2486.jpeg")
  ),
  caption: [Thermocouple against a non-conductive thermal pad (left), and secured in place (right)],
) <ssr-thermocouple>


The thermocouple works by joining two wires end-to-end, one made from chromel and the other from alumel. The difference in resistance between the two metals is used to determine the temperature at the junction. This requires running a current through the wire, and care must be taken to ensure the thermocouple probe does not contact other conductive surfaces. I insulate the probe from the boiler with a temperature conductive silicone pad.

=== Grouphead thermocouple

The grouphead heats up with the boiler and can be used to determine the degree to which the machine has pre-heated. Placing a probe on the grouphead itself is ideal because it is the primary component that the boiler water will lose heat to, as well as being the water's last point of contact with the machine before it floods the coffee grounds.

I modify the machine further: the machine front cover is disassembled, exposing the grouphead block. I place a non-conductive thermal pad on the metal and then place the thermocouple on top of that (@grouphead-thermocouple). I secure the thermocouple in place with a high-temperature polyimide film adhesive. The Type K thermocouple is attached to a second MAX31855 adaptor, which is then wired to the Pi Zero's GPIO.

#figure(
  image("../photos/IMG_2392.jpeg"),
  caption: [Type K thermocouple attached to the grouphead],
) <grouphead-thermocouple>

=== Final wiring

The Pi Zero's SPI0 interface has 2 *Chip Selects*, meaning at most two MAX31855s can be wired to that interface. The third MAX31855 can be used to connect the thermofilter, and is wired to the SPI1 interface, but this will not be used. I will leave support in place in case it is needed later. The SSR is connected to GPIO 26. This concludes the hardware modifications that have been made to the machine. A diagram showing the pinout from the Pi Zero's GPIO to the SSR and MAX31855 thermocouple amplifiers is shown in @pi-zero-pinout. The final electrical layout of the Rancilio Silvia after modifications is shown in @silvia-shelly-pi.

#figure(
  image("../diagrams/pi-zero-pinout.svg", width: 80%),
  caption: [Pi GPIO, MAX31855, #sym.amp SSR wiring diagram],
) <pi-zero-pinout>

#figure(
  image("../diagrams/silvia-shelly-pi-electrical-diagram.svg", width: 80%),
  caption: [Complete wiring diagram including all modifications],
) <silvia-shelly-pi>

#include "1-hardware-thermofilter.typ"


