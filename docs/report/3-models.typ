#set text(font: "Minion 3")
#show raw: set text(font: "Victor Mono")

= Models

== Extraction Temperature

I use the thermofilter to measure the temperature of the water exiting the grouphead in order to produce a model $f$ that will predict the extraction temperature:

$T_"Extraction" = f(T_"Boiler", T_"Grouphead")$

The extraction temperature depends on the degree to which the machine has been pre-heated. We use the grouphead temperature as a proxy variable for the pre-heating level, where the maximum observed grouphead temperature, $max(T_"Grouphead")$, defines a fully pre-heated state, and the lowest, $min(T_"Grouphead")$, defines a fully cold state.

== Temperature Lag

How long after the heat has been applied does the temperature start increasing?

== Temperature Loss

A multiple linear regression model to predicts the temperature drop for some time interval $t$ in the future under the assumption that no heat will be applied.

== Heat Level

The heat level model calculates the heat level and heat duration in order to raise the boiler temperature by $n$ #sym.degree.c.
