= Conclusion

Although the performance of the predictive model is not good enough to use practically, it demonstrates that the temperature can be held relatively stable with this approach. I am confident that with further iteration a predictive control model that is stable and substantially better than the threshold control method will emerge.

Upon completion of this project, the espresso machine now possesses advanced functionalities akin to a smart device. It can be operated remotely, allowing for power control and temperature adjustments. Real-time monitoring is available for both boiler and grouphead temperatures, alongside an accurate estimate of the extraction temperature. Additionally, there's a feature to review historical data, detailing previous shots and their respective average boiler temperatures during brewing sessions.

I have implemented the control manager such that new control methods are simple to add. Some methods that I would like to explore include a fuzzy logic system that can be defined using the known parameters of the system, such as start and stop lag time, and heat level vs temperature rise estimates. A PID neural network @shu2000pid trained using a simulated Silvia based on my dataset, which has grown to include 6,234,653 measurement observations, is a possibility.

I would also like to explore further hardware modifications, including an RTD sensor that can be mounted in a lower latency zone of the boiler @shadesofcoffeePt100Temperature, as I would like to eliminate any additional latency caused by the type K thermocouple or the insulating silicone pad. The aforementioned Pi 2 is slowly becoming available which opens up new possibilities, especially as the software is implemented with asynchronous programming paradigms, it will be ready to take advantage of multiple cores.

Software projects are never finished, and I plan to continue the development of this project until I have a predictive model that can hold the temperature stable. I have found this project a worthy challenge. Cheers #emoji.coffee.
