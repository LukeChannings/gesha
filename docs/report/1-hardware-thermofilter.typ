== Thermofilter <thermofilter>

The primary variable that I want to measure is the temperature of the water as it exits the machine and permeates the coffee: the _extraction temperature_. Understanding the relationship between the grouphead, boiler, and extraction temperatures is therefore required.

To measure the extraction temperature directly, I built a DIY measurement device called a thermofilter (a portmanteau of 'thermocouple' and 'portafilter') following specifications from a popular barista forum @diy-thermofilter. A thermofilter is a temperature sensor that is mounted to a portafilter. It is possible to measure the temperature that the coffee basket will experience by locking the thermofilter to the grouphead and pulling a shot as normal. There are commercial devices available, such as the Scace 2 @scaceThermofilter, but the cost is prohibitively high for this study.

#figure(
    image("../photos/IMG_2502.jpeg"),
    caption: [Thermofilter assembled with Pi Zero, MAX31855, #sym.amp battery. 1/2 inch scale.]
) <thermofilter-assembled>

I construct the thermofilter using a blank basket which I drill two holes into: a small hole in the centre through which water can drain to the cup, and a larger second hole off-centre that the thermocouple can be fed through. Before feeding through, I wrap the thermocouple wire in Kapton tape, which is non-conducting and temperature resistant. This will protect the wire from the sharp edges of the metal hole.

When the thermocouple is in place, I add Kapton tape (a silicone-adhesive tape that can handle very high temperatures without melting) on the exterior centre hole. I then fill the basket with a shallow layer of epoxy resin #sym.dash.em. Filling it too high will cause the epoxy to contact the grouphead's shower screen and cause water to run off the sides of the portafilter instead of into it. Before the epoxy dries I push an acupuncture needle through the centre hole in order to leave a passage for the water to drain through.

When the epoxy resin is dry, I remove the needle and tape the thermofilter to the surface of the resin to prevent it from moving during extraction (@thermofilter-basket).

In experimenting with the device, I found that there would sometimes be electrical interference from the machine when the thermofilter was connected to the Raspberry Pi. I have not thoroughly investigated the cause of this interference, however I have observed that it disappears when the thermofilter is isolated from the machine. I achieve this by using a second Pi Zero connected to a battery to gather measurements (@thermofilter-assembled).

#colbreak()

#figure(
    image("../photos/IMG_2501.jpeg", width: 50%),
    caption: "Thermofilter basket with thermofilter taped in place. 1/2 inch scale."
) <thermofilter-basket>


