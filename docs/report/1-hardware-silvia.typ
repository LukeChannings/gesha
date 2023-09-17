== Rancilio Silvia

The Rancilio Silvia has two model variations: _E_ and _M_. The _E_ model has an auto-off function that satisfies the EU's idle energy use regulations. The _M_ model omits this auto-off function and is sold outside of European markets. Both models allow the barista to brew espresso, steam milk for espresso-based drinks, and to dispense hot water through the steam wand for americanos, tea, and other hot-water beverages. In this project, an _E_ model is used.

The standard wiring of the Rancilio Silvia is shown in @silvia-e-electrical, with its stock functions described in the following sections.

#figure(
  image("../diagrams/silvia-e-electrical-diagram.svg", width: 80%),
  caption: [Rancilio Silvia E Electrical Layout],
) <silvia-e-electrical>

=== Power on

The machine is powered on when the spring-loaded power switch *IS* is toggled. This illuminates the power light *LS*, and the CPU starts a 30 minute auto-off timer, electrifying *CPU* pins 9 and 10. The wire coming in to *CPU* pin 3 will be electrified whenever the machine is in use, and it will reset the auto-off timer, thus preventing the machine from automatically powering off whilst being in use.

As soon as the machine is powered on, the boiler begins heating (see the element labelled *R*). While the element is heating, the heating light *SR* will be lit. The temperature-sensitive actuator (*TSA*) *T1* will open when the thermocouple detects a temperature greater than 100 #sym.degree.c - this will disable *R* and *SR*.

=== Brew

The barista grinds coffee into the filter basket attached to the portafilter, which is then locked to the grouphead after some additional preparation.

When the machine has been heating for a sufficient period of time and is ready to brew, the brew switch *IC* is toggled by the barista. The water pump *PO* pumps water from the water reservoir to the boiler. The solenoid valve *EG* opens, creating a path for water to pass from the boiler and into the grouphead.

The ground coffee in the filter basket provides resistance to the water which creates pressure as the pump pushes water into the boiler. The pressure forces the water through the coffee, producing espresso that falls into a cup placed underneath the portafilter.

=== Hot water and Steam

The *IA* switch is for hot water, and it effectively does the same thing as the brew switch except the solenoid valve *EG* is not opened. Hot water from the boiler is pressurised and when the steam wand valve is opened by the barista (by turning the steam knob), hot water from the boiler is pumped through the steam wand.

The *IV* switch is for steaming milk. The switch bypasses *T1*, *EG*, and *PO*, causing the *T2* TSA, rated to 140 #sym.degree.c, to control the boiler temperature. This creates pressurised steam in the boiler, which can be exhausted through the steam wand by the barista in the same manner as they dispensed hot water. Pressurised steam is used to aerate and heat milk, which creates a silky-textured hot milk that blends with the espresso.

The third TSA #sym.dash.em *T3* #sym.dash.em is a safety shut-off rated for 165 #sym.degree.c. In case the *T1* and *T2* are bypassed or broken *T3* will prevent the boiler from exceeding #sym.approx\180 #sym.degree.c.
