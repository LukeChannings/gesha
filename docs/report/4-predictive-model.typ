= Predictive Model

As I have shown in the heat session analysis, the boiler temperature will continue rise for some time after boiler heat is no longer applied (stop lag). The amount of temperature increase is related to the amount of time the boiler was heating for, and the degree to which the machine was pre-heated.

Each measurement has the following features:

- `time` - the time in UNIX epoch milliseconds
- `target_temperature` - the target temperature in degrees celsius
- `boiler_temperature` - the temperature of an espresso machine's boiler, read from a thermocouple probe mounted to the exterior of the boiler
- `grouphead_temperature` the temperature of the grouphead, the metal block to which the portafilter will be locked, which is used as a proxy variable for the machine's level of pre-heating
- `heat_level` - A number between `0` and `1`, with a step of `0.1`, which represents the amount of heat being applied to the boiler

Measurements are taken at different intervals depending on the machine's state. During an active heating session measurements are taken every 100ms, but in the idle mode records are recorded only every 1s. I used the following equation to calculate the thermal inertia carried by the water, where $I$ is thermal inertia, $Q$ is the energy input, and $#sym.Delta T$ is the rate of temperature change.

$
I = Q #sym.Delta T
$

I calculate $Q$ for each measurement by summing the heat level over a time period, which I define as being the stop lag time. In theory the stop lag time is a measure of how long the heat affects the temperature rise, and therefore in our estimate for $Q$ we should include all heat levels in that time window. A new feature, `heat_level_sum` is added to each measurement, defined as a rolling sum of heat levels for the previous records up to a maximum time interval derived from the stop lag time.

```py
    measurements["heat_level_sum"] = (measurements["heat_level"]
        .rolling(window=m, min_periods=1)
        .sum()
        .fillna(0)
    )
```

I then derive a new feature `future_boiler_temperature` by subtracting the boiler temperature $n$ measurements into the future from the current boiler temperature, creating a feature that contains the future temperature difference.

```py
measurements["boiler_temp_c_future"] = measurements["boiler_temp_c"].shift(-n)
```

I define a simple neural network using TensorFlow #sym.dash.em a commonly used library for training neural networks #sym.dash.em to predict `boiler_temp_c_future` from `grouphead_temp`, `boiler_temp`, and `heat_level_sum`. I define a model with 2 hidden layers with 64 and 8 units respectively. I use the tanh activation function for the hidden layers and a linear activation for the output. I train for 50 epochs and a batch size of 128. There are 46 million observations total, I reserve 33% for testing the model.


In this first model I use $m = 260$ and $n = 2160$, which were not derived from the stop lag time. Instead, these coefficients were found by searching permutations of $m$ and $n$ such that the Pearson correlation coefficient between `heat_level_sum` and `boiler_temp_c_future` was the highest.

The fit model has a mean squared error (MSE) of 49 and an $R^2 = 0.75$. The model is not very well fit, but may show interesting results. I convert the model to ONNX format (visualised in @onnx-predictive) and validate the inference results are consistent between Tensorflow and the ONNX runtime.

#figure(
    image("../diagrams/predictive-10-09-2023.svg"),
    caption: [A graph representation of the neural network exported to ONNX]
) <onnx-predictive>

To integrate the ONNX model into the Rust application code, I implement a module to expose the prediction function `predict_boiler_temp_diff`. I update the controller manager to keep a rolling sum of heat levels and update the definition of the `sample` function to include the rolling sum.

I implement the `sample` function for the predictive control method:

```rs
    fn sample(&mut self, boiler_temp_c: f32, grouphead_temp_c: f32, heat_level_sum: f32) -> f32 {
        let predicted_boiler_temp = self.model.predict_boiler_temp_diff(
            grouphead_temp_c,
            boiler_temp_c,
            heat_level_sum,
        );

        ...

        let heat_level = if predicted_boiler_temp > self.target_temperature {
            0.0
        } else {
            1.0
        };

        heat_level
    }
```

@predictive-control-test shows a real-world test of the model. The model is attempting to hold the boiler temperature at 100#sym.degree.c over a 2.25 hour period between 11:30am and 2:15pm. The red vertical bars represent where the model turned on the boiler, whose temperature is represented by the blue line. The orange line represents the grouphead temperature, and the red dashed line the target temperature. As apparent, the prediction is not stable. Although it accurately identifies points where the non-heating water will decrease in temperature, it does not allow the temperature drop even when the water is higher than the target. In these cases, the model turns the boiler on anyway to combat the temperature drop, and just moves the boiler temperature further away from the target. It also has periods were it allows the temperature to drop well below the target without a rapid attempt to correct it. Notice too that the grouphead temperature has much more temperature variation here than it did in the threshold control method described earlier.


Red vertical lines indicate the boiler heat being turned on. The predictions are not stable, and often predict a temperature drop despite the temperature being well above the target. The model does keep the temperature relatively close to the target temperature (shown by the red horizontal dashed line), but would also drive the heat very high (see around the 14:00 interval).

#figure(
    image("../diagrams/predictive-model-test-1.svg"),
    caption: [Predictive control test]
) <predictive-control-test>

I theorise that because the model was trained on the entire measurement set, and the temperature decreases for the majority of observations (as the machine is idle or cooling most of the time), the model has fit on these measurements instead of the more important heat session measurements.

My next steps for producing a more reliable model would be to fit a new model only on measurements that are within a heat session, with a time margin before and after so that the heat level sum has a gradual increase. I would do more analysis into heat sessions in order to remove sessions that are not a good representation of the relationship between heat output and temperature difference. Ensuring the quality of the training data whilst also retaining enough training data to fit a model that will generalise is possible, but may require more measurements.
I have implemented a Python API that enables the automation of experiments, which I originally used in my analysis of the boiler heat level responses. (Code reference: #link("https://github.com/Birkbeck/msc-project-source-code-files-22-23-LukeChannings/blob/main/experiments/repeatable/power_level/__main__.py")[`experiments/repeatable/power_level/__main__.py`]). Further such experiments will help to produce more self-contained measurement samples that can be used for model training. Ideally all measurement data can be utilised, and work will be undertaken to filter out low quality measurements.

By augmenting the measurement data with the `heat_level_sum` and `future_boiler_temperature` features I have manually created new features that I had hoped would capture the desired relationship. This may have been too simple, and the relationship may be learnt directly instead of through manual intervention. If the model produced from the heat session measurements also performs poorly I will explore a time series long short term memory (LSTM) architecture that may be able to capture the relationship between the heat input and temperature rise. Such an architecture could fit on patterns in time series data, which our measurement data undoubtedly is. This could allow a model to be trained on a series of measurements for an interval, and find patterns within this series. A larger model could also include more features such as `brew` and `steam`, as well as `boiler_temperature`, `grouphead_temperature` and `heat_level`.

Such an architecture would present new challenges for integrating the model into the main application though, as the reason a simple model was initially chosen was to make inference easy and fast. An LSTM model necessitates a 3-dimensional structure that I have experimented with for a new extraction temperature model, but have had difficulty integrating into the main application because of the more complicated conversion to ONNX format. As well as integrating the model and performing inference the software will have to store a large number of measurements in memory to be used for model input. This additional overhead may cause problems for sub-100ms inference on the Pi.
