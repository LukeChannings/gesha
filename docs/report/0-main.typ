#set page(margin: 0.8in)
#set par(leading: 0.55em, justify: true, first-line-indent: 0.5em)
#set text(font: "Minion 3")
#show raw: set text(font: "Victor Mono")
#show heading: set block(above: 2em, below: 1.3em)
#show heading: set text(font: "Minion 3 Subhead")

#show figure: it => align(center)[
#it.body
#text(font: "Minion 3 Caption", weight: 800, size: 9pt)[
    #emph[
        #it.supplement
        #it.counter.display(it.numbering)
    ]
    #sym.dash.en #it.caption
]
]

#set document(
    title: "Temperature Stability in Consumer-Grade Semi-Automatic Espresso Machines",
    author: "Luke Channings"
)

#align(center, block(width: 85%, below: 5em)[
    #text(size: 16pt)[
        *Temperature Stability in Consumer-Grade Semi-Automatic Espresso Machines*
    ]

    #text(size: 14pt)[Luke Channings]

    #set par(leading: 0.7em)

    #text(size: 11pt)[
        A dissertation submitted in partial fulfillment of the requirements for the \
        MSc in Advanced Computing Technologies
    ]

    #text(size: 9pt, baseline: 2em)[
        Department of Computer Science and Information Systems \
        Birkbeck College, University of London \
        September 2023
    ]
])

#box()[
  #align(center)[*Abstract*] \
  Extraction temperature #sym.dash.em the temperature of boiler water when it meets the ground coffee #sym.dash.em is an important factor in producing a well balanced espresso with consistent results. A temperature in the range of 90#sym.dash.en\100 #sym.degree.c is desirable.

    I modify a common home espresso machine #sym.dash.em the Rancilio Silvia #sym.dash.em to enable the gathering of quantitative temperature data. I develop software to record and store these data. I produce predictive models for aspects of the machine's behaviour, and I combine them to define a new temperature control method.
]


#outline(title: "Table of Contents", indent: 1.5em)

= Introduction

The Rancilio Silvia's temperature control method uses a Temperature-Sensitive Actuator (TSA) to control the boiler: when the boiler temperature is #sym.lt 100 #sym.degree.c the boiler is heating; when it's #sym.gt.eq.slant 100 #sym.degree.c it isn't.

#figure(
    image("../../models/stock_control/boiler_temp_sawtooth.svg", width: 90%),
    caption: [100 #sym.degree.c Threshold Control Method],
) <threshold-control>

#columns(2, gutter: 8pt)[
There are several problems with this mechanism:

1. The actual brew temperature isn't consistent, it could be as low as 70 #sym.degree.c or as high as 100 #sym.degree.c depending on the state of the machine (_consistency_).
2. There is no way to know what the current temperature is (_observability_)
3. There is no way to adjust the desired temperature (_configurability_)

@threshold-control shows the characteristic sawtooth pattern that results from the default control method. The pink rectangles show when the heat is being applied. The time needed to bring the temperature back above 100 #sym.degree.c is a function of the machine's thermal saturation (pre-heating) #sym.dash.em notice that it tends to decrease over time. The boiler is not immediately responsive to the applied heat, which results in the temperature overshooting the set point by 8 #sym.degree.c on average.

#colbreak()

In this report I will explain the changes I have made to the stock Rancilio Silvia to provide a hardware platform suitable for developing a software solution to this problem. I will use real-time temperature measurements and predictive models to target a specific extraction temperature. The software development process is cyclical, I record measurements from the machine and analyse those measurements to produce models that I can then feed back into the software.

I will then describe the software architecture, how measurements are recorded and queried, how custom temperature control methods are implemented, and how the user can interact with the application. I will present predictive models and quantitative analysis that can be used to inform the predictive control system as well as enhance the user interface. Finally I will present a predictive control method that provides _consistency_, _observability_, and _configurability_ of the brew temperature.
]


#include "1-hardware.typ"
#include "2-software.typ"
#include "3-models.typ"
#include "4-predictive-control.typ"

#bibliography("citations.bib")
