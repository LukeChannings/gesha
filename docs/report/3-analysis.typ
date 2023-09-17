#set text(font: "Minion 3")
#show raw: set text(font: "Victor Mono")

= Analysis

I have accumulated 5.5 million observations of the machine. When the machine is idle these measurements are taken every second, and when the machine is actively heating they are taken every 100ms, but only when the boiler or grouphead temperature changes. I use these observations to learn about the behaviour of the machine.

== Pre-heating

As discussed in the introduction, the degree to which a machine has been pre-heated is a factor in many important aspects of the machine. In this section I will discuss the quantitative measure of determining the machine's pre-heating level, i.e. whether the machine has been fully pre-heated. This is an important factor to quantify, since it impacts the extraction temperature and therefore the taste of the espresso.

The machine could be said to be preheated when $T_"Grouphead" / max(T_"Grouphead") = 1$, where $T_"Grouphead"$ is the grouphead temperature, but @target-temp-vs-grouphead shows that the $T_"Grouphead"$ changes in response to the target temperature, and so will vary for all target temperatures. In the figure, you can see that the target temperature was changed from 105#sym.degree.c to 95#sym.degree.c at approximately 14:55. The machine recalibrates, and continues with its sawtooth pattern at the lower temperature.

#figure(
    image("../diagrams/better-chart-1693405727116-1693408802116.svg", width: 80%),
    caption: [Grouphead (blue) and boiler (red) temperatures response to a target temperature change (dashed black)]
) <target-temp-vs-grouphead>

Another reason that using the maximum value would produce inaccurate results is that the grouphead temperature can be very high when the machine is in steam mode, since $T_"Boiler"$ is often #sym.gt 140 #sym.degree.c. Current database records show that the maximum value for $T_"Grouphead"$ is 108.5 #sym.degree.c.

The heuristic I have chosen is to find a piecewise function that for each target temperature will return the equilibrium grouphead temperature. Notice in @target-temp-vs-grouphead that $T_"Grouphead"$ stabilises over time. By taking the _modal_ value of $T_"Grouphead"$ for each $T_"Target"$ we can derive our piecewise function (@preheat-heuristic).

#figure(
    $
    f(t) := cases(
        74.0 "if" t <= 90.0 "else" \
        76.0 "if" t <= 93.0 "else" \
        78.0 "if" t <= 95.0 "else" \
        80.0 "if" t <= 99.0 "else" \
        82.0 "if" t <= 101.0 "else" \
        84.0 "if" t <= 103.0 "else" \
        86.0 "if" t <= 107.0 "else" \
        88.0 "if" t <= 109.0 "else" \
        90.0
    )
    $,
    caption: [$f(t)$, where f is the heat level function and $t$ is the target temperature]
) <preheat-heuristic>

The piecewise function can be used to determine that the machine has fully pre-heated with the following:

$min(T_"Grouphead" / f(T_"Target"), 1) = 1$

The function $f(t)$ will become more accurate over time, since its coefficients are derived by querying the measurement database (@heat-level-query). As such, re-running the query and updating the coefficients as more target temperatures are selected will further refine the coefficients.

#figure(
```sql
    WITH RoundedGroupheadTemperatures AS (
        SELECT
            target_temp_c,
            ROUND(grouphead_temp_c / 2) * 2 as rounded_grouphead_temp_c
        FROM measurement
        AND power IS TRUE AND STEAM IS FALSE AND pull IS FALSE
    ),

    TemperatureCounts AS (
        SELECT
            target_temp_c,
            rounded_grouphead_temp_c,
            COUNT(*) as count
        FROM RoundedGroupheadTemperatures
        GROUP BY target_temp_c, rounded_grouphead_temp_c
    ),

    ModalTemperatures AS (
        SELECT
            target_temp_c,
            MAX(count) as max_count
        FROM TemperatureCounts
        GROUP BY target_temp_c
    )

    SELECT
        m.target_temp_c,
        t.rounded_grouphead_temp_c as modal_grouphead_temp_c
    FROM ModalTemperatures m
    JOIN TemperatureCounts t ON m.target_temp_c = t.target_temp_c AND m.max_count = t.count
    ORDER BY m.target_temp_c;
```,
caption: [SQL query to find the modal $T_"Grouphead"$ for $T_"Target"$]
) <heat-level-query>

It is also possible to rely on simply measuring how long the machine has been heating, however this method is not effective in a scenario where the machine is pre-heated fully for the first session and partially cooled for a second session. The pre-heating time will be shorter when the machine has not fully cooled down.

== Boiler Power vs Temperature Increase <model-boiler-power-vs-temperature>

The threshold control method (descibed in the Introduction) yields poor results with regards to the accuracy of the temperature for two main reasons: the temperature reading from the boiler thermocouple does not reflect the true temperature of the boiler, it is attached to the outside case of the boiler, and not directly to the element itself. As such there is a lag between the application of heat in the boiler and the measurement of the results of this heat.

In this model I gather data that captures the relationship between the application of heat and the temperature change that we read. I will show that the pre-heat level of the machine is a dependent variable.

#figure(
    grid(columns: (1fr, 1fr),
    image("../diagrams/heat-level-15s.svg", width: 100%),
    image("../diagrams/heat-level-30s.svg", width: 100%)
    ),
    caption: [Temperature response to heat levels 0 #sym.dash.em 1]
) <heat-levels>

== Temperature Loss

The machine will cool when the boiler is not engaged. A component of the predictive model is the modelling of temperature loss, so that an appropriate heat time and duration can be executed to counteract the loss of heat. I define a query (@cooling-down-query) to retrieve measurements  from the machine as it cools down. I will use the results as a basis for a multiple linear regression model that can predict the future temperature in the absence of heating.

#figure(
```sql
SELECT time, boiler_temp_c, grouphead_temp_c
FROM (
    SELECT time,
        LAG(boiler_temp_c) OVER () boiler_temp_c_prev,
        LAG(time) OVER () time_prev,
        boiler_temp_c,
        grouphead_temp_c,
        LAG(grouphead_temp_c) OVER () grouphead_temp_c_prev,
        heat_level,
        pull,
        power
    from measurement
) AS inner_query
WHERE boiler_temp_c_prev >= boiler_temp_c
    AND grouphead_temp_c_prev >= grouphead_temp_c
    AND heat_level = 0.0
    AND pull = FALSE
    AND power = FALSE
ORDER BY time ASC;
```,
    caption: [SQL query to retrieve measurements where the machine is cooling down]
) <cooling-down-query>


#figure(
    image("../diagrams/thermal-loss-model.svg", width: 100%),
    caption: [Temperature response to heat levels 0 #sym.dash.em 1]
) <temperature-loss>

The model takes an input of the current boiler and grouphead temperature and produces a predicted temperature loss for each after 5 minutes.

$
f(T_"Boiler", T_"Grouphead") = #sym.Delta ( T_"Boiler", T_"Grouphead" )
$

I arrived at 5 minutes as a time period by comparing the performance of 9 models, using the periods: $10s, 30s, 1m, 5m, 10m, 20m, 30m, 1h$. The models were trained using cross-validation with $2/3$ training data and $1/3$ testing.

The larger the time interval, the better the $R^2$ and $"MSE"$ tended to be. However, the shorter the time interval is, the more useful the predictive model is. I found the 5 minute model was a good compromise between utility and accuracy.

@temperature-loss shows the results of the 5 minute model predicting temperature loss with initial parameters of $T_"Boiler" = 92 #sym.degree.c, T_"Grouphead" = 75 #sym.degree.c$.

== Heat sessions

#include "3-analysis-heat-sessions.typ"

== Extraction Temperature

Extraction Temperature ($T_"Extraction"$) is the temperature of the water as it comes out of the espresso machine and into the portafilter. This is the temperature that I am interested in, not the boiler, because it is the temperature of the water as it meets the coffee and subsequently the temperature that has an effect on the coffee's taste.

Because observations are time consuming to gather there are relatively few observations available. My process for gathering observations is as follows:

1. Lock the thermofilter into the grouphead,
2. Actively record thermofilter measurements on the secondary Pi Zero, recording a timestamp and the current grouphead temperature
3. Tap the Gesha UI brew button and flip the brew switch simultaneously,
4. Observe the thermofilter measurements in the UI #sym.dash.em they are broadcast from the secondary Pi Zero over MQTT and integrated into the UI
5. Flip the brew switch back when the thermofilter temperature dovetails with the boiler temperature

I follow this procedure repeatedly, varying the target temperatures and the pre-heat level. The thermofilter is emptied and the grouphead is flushed between sessions. I combine all observations and filter them by the shot `start_time` and `end_time` (@shots). I have $41$ experiments, and a combined total of $10,043$ observations.

#block(
    width: 100%,
    height: 100%,
    breakable: false,
    [
        #figure(
            image("../diagrams/shots-with-extraction-temperature.svg", width: 100%),
            caption: [All extraction temperature measurements]
        ) <shots>
    ]
)
The Pearson correlation coefficient ($r$) is a measure of the linear relationship between two variables. The coefficient has a range of $-1$ to $1$, where $1$ or $-1$ are a perfect (positive and negative, respectively) linear relationship between two variables and $0$ represents variables with no correlative relationship. @corr-heatmap shows a heatmap of the coefficients for $T_"Grouphead"$, $T_"Boiler"$, and $T_"Extraction"$. The $r$ with respect to $T_"Grouphead"$ and $T_"Extraction"$ is $0.8$, and $r$ with respect to $T_"Boiler"$ and $T_"Extraction"$ is also $0.8$.

#figure(
    image("../diagrams/extraction-temperature-correlation.svg", width: 60%),
    caption: [Pearson correlation heatmap for $T_"Grouphead"$, $T_"Boiler"$, and $T_"Extraction"$]
) <corr-heatmap>

An $r$ value of $0.8$ suggests a moderate to high linear correlation with the extraction temperature. Each model was trained with 80% of the data, with 20% reserved for testing. More data in more varied conditions will improve the model further, and this model in particular will benefit from more accuracy.

First I fit a multiple linear regression model using $T_"Grouphead"$ and $T_"Boiler"$ to predict $T_"Extraction"$. This produced a model with $R^2 = 0.843$ and a $"MSE" = 31.5$, $sqrt("MSE") = 5.5 degree.c$.

Next I trained a Support Vector Machine (SVM) with the Radial Basis Function to test if a non-linear regression performed better. I used a Grid Search Cross Validation and found the coefficients $gamma = 1.0$ and $C = 1000$ to perform the best. Training a model with these parameters resulted in a better fitting model with $R^2 = 0.935$ and $"MSE" = 13$, $sqrt("MSE") = 3.6 degree.c$.

Finally, I trained a neural network using Tensorflow, starting with a single layer network with 16 units and experimenting with the number of units and hidden layers until a well-performing model was produced. I used a tanh activation function and a linear output. The training data was not rescaled because the inputs and outputs are all in the same unit (degrees celsius). I trained the model using an $"MSE"$ loss function.

I produced a simple neural network model with $R^2 = 0.96$ and $"MSE" = 7.6$, $sqrt("MSE") = 2.75 degree.c$ (@extraction-temp-nn). This is the best performing model. I integrate this model into the Gesha application, publishing a predicted value on the `temperature/thermofilter_predicted` topic.
This is rendered in the UI in real time along with the boiler and grouphead temperatures, providing a useful prediction to the barista.

#figure(
    image("../diagrams/extraction_temperature_model.svg"),
    caption: [A graph of the extraction temperature neural network]
) <extraction-temp-nn>

= Inference on the Pi Zero

Machine Learning models are typically run in production using the same software stack that they were trained with (e.g. Python + Tensorflow, PyTorch, XGBoost, etc). Containers are often used to make this process simpler and less error prone. My application has a unique set of constraints: the model must produce an inference result within 100ms whilst running on a low-power device (the Pi Zero has a 1GHz single core CPU and 512MB RAM), so that the information can guide a barista in real-time. Whilst there are container runtimes under Linux with very low overhead, with runc being a good choice @espe2020performance, there are additional complications with software availability for 32-bit ARM. Furthermore, I benchmarked the time required run a "hello world" script on the Pi Zero with no other software running #sym.dash.em it took 272ms.

To account for these constraints, I looked into running the model within the Rust application itself. This would provide a large speed improvement since Rust is close to C in terms of performance, and also because any overhead incurred by starting a sub-process will be avoided. The Open Neural Network Exchange (ONNX) @bai2019 is a format that models the network as a directed acyclic graph (DAG). The nodes in the graph represent specific mathematical operations from simple addition or matrix multiplication to long short-term memory (LSTM) recurrent neural networks. Edges describe the inputs and outputs between these nodes, and "initialisers" can be used to store data, like trained weights and biases, within nodes.

It is possible to convert a Tensorflow model to ONNX, which is serialised using protocol buffers (protobuf) @protobuf, a binary format invented at Google. In order to run inference with the model, an ONNX runtime is required. The runtime will parse the protobuf file into the ONNX DAG. The runtime will need to implement the operators that are defined in the ONNX specification and allow the user to pass in data to the input node, and return the result produced in the output node of the network.

Tract is an ONNX runtime, or "inference engine" that is developed by Sonos @sonosOptimisingNeural, originally to support wake word recognition on its low-performance speakers. Tract is implemented in Rust, which makes it ideal for this use case. An alternative runtime that implements a larger set of operators is ONNX runtime. I chose not to use this because it is a C++ library and does not have an official Rust module, which makes integration more involved.

The downside to this approach is that models converted from Tensorflow to ONNX do not always work, either due to a lack of operator support in the runtime (meaning the model won't run at all) or a difference in the weight types (`float32` vs `float64`, for example) resulting in errors in inference (the inference result is an order of magnitude different from the Tensorflow inference result). As such I have kept my models as simple as possible, relying on optimisation of training data instead of more complex models.
