== Thermofilter

A thermofilter is a temperature sensor that is mounted to a portafilter. It is possible to measure the temperature that the coffee basket will experience by locking the thermofilter to the grouphead and pulling a shot as normal.

#figure(
    image("../photos/IMG_2502.jpeg"),
    caption: [Thermofilter assembled with Pi Zero, MAX31855, #sym.amp battery]
) <thermofilter-assembled>

#columns(2, gutter: 8pt)[

I construct the thermofilter using a blank basket which I drill two holes into: a small hole in the centre through which water can escape, and a larger second hole off-centre that the thermocouple can be fed through. Before feeding through I wrap the thermocouple wire in Kapton tape, which is non-conducting and temperature resistant #sym.dash.em this will protect the wire from being cut into by the edge of the metal hole.

When the thermocouple is in place I place Kapton tape on the exterior centre hole. I then fill the basket shallowly with epoxy resin #sym.dash.em filling it too high will cause the epoxy to contact the grouphead's shower screen. Before the epoxy dries I place an accupuncture needle through the centre hole in order to leave a passage for water to escape through.

When the epoxy resin is dried I tape the thermofilter to the surface of the resin to prevent it from moving during extraction (@thermofilter-basket).

#colbreak()

#figure(
    image("../photos/IMG_2501.jpeg"),
    caption: "Thermofilter basket with thermofilter taped in place"
) <thermofilter-basket>
]

In my experiments I found that there would sometimes be electrical interference from the machine when the thermofilter was connected to the Raspberry Pi. I have not thoroughly investigated the cause of this interference, however I have observed that it disappears when the thermofilter is isolated from the machine. I use a second Pi Zero connected to a battery to gather measurements (@thermofilter-assembled).
