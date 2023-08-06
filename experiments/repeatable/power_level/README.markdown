# Power Level experiment

This experiment is to learn the temperature change in degrees C for all heat levels from 0-100%, in 10% increments.

The data will include:

-   **Time** (Unix Epoch Millis): The time when the heat was first applied
-   **Duration** (Millis): The amount of time the heat was applied for
-   **Level** (0,0.1, ..., 0.9, 1.0): the heat level of the boiler
-   **LagMin** (Millis): the amount of time after heat first being applied for the temperature to start increasing
-   **LagMax** (Millis): the amount of time between the heat stopping and the temperature reaching its maximum
