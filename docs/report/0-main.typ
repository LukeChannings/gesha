#set page(margin: 0.8in)
#set par(leading: 0.55em, justify: true, first-line-indent: 0.5em)
#set text(font: "Minion 3")
#show raw: set text(font: "Victor Mono")
#show heading: set block(above: 2em, below: 1.3em)
#show heading: set text(font: "Minion 3 Subhead")

#show figure: it => align(center)[
#block(outset: 3em, breakable: false, [
#it.body
#text(font: "Minion 3 Caption", weight: 800, size: 9pt)[
    #it.caption
]
])
]

#set heading(numbering: "1.")

#set document(
    title: "Temperature Stability in Consumer-Grade Semi-Automatic Espresso Machines",
    author: "Luke Channings"
)

#align(center + horizon, block(width: 85%, height: 100%, breakable: false)[
    #set par(leading: 0.7em)

    #align(center, text(15pt)[

    *Temperature Stability in Consumer-Grade Semi-Automatic Espresso Machines*

    #pad(top: 1.5em, bottom: 1.5em,
    text(size: 12pt)[
        A dissertation submitted in partial fulfilment of the requirements for the MSc in Advanced Computing Technologies
    ]
    )

    #text(size: 12pt)[by Luke Channings]

    #text(size: 12pt, baseline: 2em)[
        Department of Computer Science and Information Systems \
        Birkbeck, University of London \
        September 2023
    ]
    ])
])

#pagebreak()

#align(left + horizon)[
#set par(leading: 0.7em)
This report is substantially the result of my own work except where explicitly indicated in the text. I give my permission for it to be submitted to the TURNITIN Plagiarism Detection Service.
I have read and understood the sections on plagiarism in the Programme Handbook and the College website.

The report may be freely copied and distributed provided the source is explicitly
acknowledged.
]

#pagebreak()

#box()[
  #align(center)[*Abstract*] \
 Consumer-grade semi-automatic espresso machines are affordable appliances for making espresso at home. They allow a home barista to brew espresso, steam milk, and produce hot water. Producing a consistent and delicious tasting espresso at home is not easy and takes practice. Developing a profile for a perfect shot of espresso, known as "dialing in" the shot, requires an iterative process of weighing beans, grinding them to a specific consistency, evenly distributing the ground coffee in the basket, compressing the grinds evenly and finally pumping hot water through the grinds. Each part of this process needs to be optimised by the barista, and a more detailed and specific set of criteria will produce consistently delicious espresso.

Extraction temperature is the temperature of the hot water that is pumped through the ground coffee, and it is an important factor in producing a well balanced espresso with consistent results. An extraction temperature in the range of 90-100 #sym.degree.c is desirable. In this study, I use a machine-learning approach to measure, quantify, predict, and implement temperature control in a common home espreso machine. The increased control, when achieved, will allow the barista better information about the characteristics of their espresso-brewing process, but also allow them to optimize their process going forward.
]


#outline(title: "Table of Contents", indent: 1.5em, depth: 2)

#pagebreak()

#outline(
  title: [List of Figures],
  target: figure,
)

#pagebreak()


#include "0-introduction.typ"
#include "1-hardware.typ"
#include "2-software.typ"
#include "3-analysis.typ"
#include "4-predictive-model.typ"
#include "5-conclusion.typ"

#bibliography("citations.bib")
