There are a large number of measurements (more than 6 million), most of which are not useful for making predictions of the extraction temperature because they record the machine in an idle or cooling state. However, the identification and categorisation of these measurements is important for understanding the behaviour of the machine. A heat session is a discrete period of time where the boiler is active and heating. The period of time in between the application of heat and the temperature rise caused by this heat it reaching its peak defines the bounds of a heat session.

I implemented a multi-step procedure for identifying a range of measurements that belong to a specific heat session. Note that if two heat sessions are close together they will be merged, which muddies the analysis.

*Contiguous measurement groups*

There are occasional breaks in the heat session measurements (a discontinuity). This may be where the software was restarting, a bug in the software, or a loss of power. The first step is to identify contiguous groups of measurements in order to avoid incorrect analysis caused by a discontinuity. Measurements are only recorded when there has been a change in the boiler or grouphead temperatures. The sensors are read from every 100ms, this means that it is possible to rescale the recorded measurements to a precision of 100ms by back-filling the value from the previous measurement, since I know that the temperature has not changed in the intervening period. This presents a problem though, as there may be discontinuities where measurements were not recorded for reasons other than the temperature being stable.

#figure(
    $
    abs( T_"Boiler"_i - T_"Boiler"_(i+1) ) gt 2.0 degree.c
    $,
    caption: [The last measurement in a contiguous group is determined when the temperature difference to the next measurement is more than 2 #sym.degree.c]
)

I analysed the measurements and determined that a temperature rise of 2 degrees is possible within a 100ms period, but any higher than this is a discontinuity. It is not as reliable to apply such a heuristic directly to the timestamp because when the temperature is stable during idle, it is possible that no measurements are recorded for large periods (hours), making it impossible to determine discontinuity using the timestamp.

_Code reference: #link("https://github.com/Birkbeck/msc-project-source-code-files-22-23-LukeChannings/blob/main/predictive/dataset.py")[predictive/dataset.py\#group_by_contiguous_measurements]_

*Heat session bounds*

For every contiguous group of measurements, I must find the subset of measurements where heat was being applied and the temperature consequently increased. To determine the start of a heat session I find the indices for measurements where the heat level (a value between 0 and 1, with intervals of 0.1, indicating the level of heat the boiler is outputting) goes from $0$ to $gt 0$, followed by measurements remain $gt 0$ within a threshold. The threshold is defined because there are periods of time where the heat level will osculate between 0 and 1 and this does not indicate the heat session has ended. There is a single iteration through the measurements to determine the bounds, this prevents heat sessions from overlapping.

_Code reference: #link("https://github.com/Birkbeck/msc-project-source-code-files-22-23-LukeChannings/blob/main/predictive/dataset.py")[predictive/dataset.py\#get_heat_session_bounds]_

*Start lag* and *Stop lag*

When the boiler heating element is engaged, the heat is instantly applied to the water in the boiler. However the resulting temperature rise does not propagate to the boiler temperature sensor for some period of time - this lag time is called the start lag. To measure the start lag we measure the time between the start of the heat session and the first measured temperature increase.

Similarly, when the heating element is disengaged the temperature does not immediately stop rising due to accumulated thermal inertia - I call this the stop lag. I measure the stop lag as the amount of time between the heat level going to 0 and the boiler temperature reaching a plateau.

_Code reference: #link("https://github.com/Birkbeck/msc-project-source-code-files-22-23-LukeChannings/blob/main/predictive/dataset.py")[predictive/dataset.py\#get_heat_session_summaries]_

*Heat session anatomy*

Heat sessions are not always easy to separate from the measurement data, so I define some characteristics that identify bad data. Heat sessions with an instant start lag indicate that there was already thermal inertia that caused the temperature to increase, making this heat session inaccurate. Sessions with an immediate stop lag, or an exceedingly long stop lag are also excluded. Finally, sessions with a starting boiler temperature of $> 100 degree.c$ are excluded. @heat-session-anatomy shows the ratios for a mean average heat session. The colored bar represents the stats of the boiler, while the dotted markers indicate where the output data registers a change.

#figure(
    image("../diagrams/heat-session-anatomy.svg"),
    caption: [Heat session anatomy]
) <heat-session-anatomy>

@heat-session-summary shows summary statistics for the heat sessions. Further work could be done to remove outliers, including the performance of automated experiments to generate measurements that are easier to separate. It is important to have a reasonably large sample of observations, however, as such the choice of parameters for culling outliers is conservative.

#figure(
    table(columns: 7,
[], [*Session duration* (seconds)], [*Total heat* (seconds)], [*Start lag* (seconds)], [*Stop lag* (seconds)], [*Starting boiler temperature* (#sym.degree.c)], [*Temperature* #sym.Delta (#sym.degree.c)],
[count], [533], [533], [533], [533], [533], [533],
[mean], [93.37], [21.20], [7.95], [48.74], [93.33], [10.78],
[std], [14.45], [16.10], [5.58], [9.74], [9.73], [8.63],
[min], [37.10], [1.78], [2.00], [25.40], [24.25], [0.75],
[25% (Q1)], [87.40], [15.44], [3.00], [43.30], [90.00], [8.25],
[50% (Q2)], [91.30], [16.89], [5.30], [46.20], [95.00], [9.00],
[75% (Q3)], [96.40], [20.24], [13.50], [50.50], [100.00], [10.75],
[max], [214.30], [130.39], [19.90], [89.40], [100.00], [81.25],
),
caption: [Summary statistics for the heat sessions]
) <heat-session-summary>
