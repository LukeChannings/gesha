== Rancilio Silvia

The Rancilio Silvia has two model variations. The _E_ model, which has an auto-off function that satisfies the EU's idle energy use regulations. The second is the _M_ model, which omits this auto-off function and is sold outside of European markets. All models support three basic operations: _Brew coffee_, _Steam milk_, and _Dispense hot water_.

I will explain how the machine's electronics work when performing these three operations, and later I will show how I have modified the electrical layout (@silvia-e-electrical) to create a hardware platform to support my application.

#figure(
  image("../diagrams/silvia-e-electrical-diagram.svg", width: 80%),
  caption: [Rancilio Silvia E Electrical Layout],
) <silvia-e-electrical>

#columns(2, gutter: 16pt, [
=== Power on

The machine is powered on when the spring-loaded power switch *IS* is toggled. This illimunates the power light *LS*, and the CPU starts a 30 minute auto-off timer, electrifying *CPU* pins 9 and 10. The wire coming in to *CPU* pin 3 will be electrified whenever the machine is in use, and it will delay the auto-off timer.

As soon as the machine is powered on the boiler begins heating (see the element labelled *R*). Whilst the element is heating the heating light *SR* will be lit. The temperature-sensitive actuator (TSA) *T1* will open when the thermocouple detects a temperature greater than 100 #sym.degree.c - this will disable *R* and *SR*.

=== Brew

When the machine is warmed up and ready to brew the brew switch *IC* may be toggled by the user. The water pump *PO* pumps water from the water reservoir to the boiler. The solenoid valve *EG* opens, creating a path for water to pass from the boiler and into the _grouphead_.

Coffee is ground into a _basket_ that is mounted to a _portafilter_ #sym.dash.em a portmantau of _portable_ and _filter_, it is the  #sym.dash.em, which is locked to the group head after preperation. The coffee provides resistance to the water which creates pressure as the pump pushes water into the boiler. The pressure forces the water the ground coffee producing espresso.

=== Hot water and Steam

The *IA* switch effectively does the same thing as the brew switch, except the solenoid valve *EG* is not opened, meaning the hot water being pumped through the boiler goes out via the steam wand.

The *IV* switch bypasses *T1*, *EG*, and *PO*, thus using the *T2* TSA, which is rated to 140 #sym.degree.c. This evaporates the water in the boiler, creating steam, which is then expelled with a manual valve release knob.

The third TSA #sym.dash.em *T3* #sym.dash.em is a safety switch that will open when the boiler temperature exceeds 165 #sym.degree.c.
])
