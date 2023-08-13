# Thermal loss

At what rate does the temperature approach ambient temperature? This is non-linear - can we predict the temperature loss?

## Dataset

The data I am looking for is simply a series of measurements where the boiler temperature drops after the machine has been used. An example situation is after the machine has been heated for usage, starting from when the temperature begins to drop and ending when the temperature reaches ambient room temperature.

I have a query to find all temperature measurements where the *boiler temperature* and *grouphead temperature* decrease over time using this [SQL query](query.sql).

The measurements should be recorded at most 1 second apart, however observed measurements can be up to 10 seconds apart, his is likely caused by unreliability inherent in the mechanism for recording or taking measurements, I have not spent time investigating since the problem can be remedied with processing.

## Data processing

I group measurements that are contiguous, i.e. taken at most 10 seconds apart from one another.

I further filter the groups:

- The temperature must drop by at least 50 &deg;C. This would happen if the boiler starts at 80 &deg;C and drops to 30 &deg;C, for example.

I then trim the data to ensure the first observation includes the highest temperature.

I then resample the data to an arbitrary time increment, decided at runtime, so that observations over seconds are combined into observations over minutes or hours, etc. This alleviates some problematic features of the raw measurements, such as oscillating temperature increments caused by the relatively low resolution (0.25 &deg;C) of the sensors.


## Modelling

The model takes $X_{Boiler Temp}$ and $X_{Grouphead Temp}$ and produces the expected change in temperature of the same: $Y_{Boiler Temp Diff}$ and $Y_{Grouphead Temp Diff}$.

To experiment to find the ideal resample time increment, I train several Multiple Linear Regression models to find the best fit model with the smallest time increment.

|Resample Time|R2|MSE|
|-|-|-|
|min|0.036196|0.007702|
|10S|0.110058|0.008753|
|30S|0.448490|0.011952|
|1T|0.660993|0.018699|
|5T|0.908268|0.087685|
|10T|0.906895|0.347437|
|20T|0.931177|0.887888|
|30T|0.926796|1.373038|
|1H|0.854855|5.610742|

\* "min" is computed for each group, it's the highest time delta between two "contiguous" measurements, where the threshold for a measurement to be considered contiguous is a maximum of 10 seconds. 

**5T** is a model that predicts the next temperature in 5 second increments. The model is trained on 66% of the dataset and validated with 33%, a common cross-validation technique. This model has the highest $R^2$ value and a reasonably low $MSE$.

This model will be used to estimate the rate at which the temperature drops in order to counteract the temperature loss. At the moment a simple Multiple Linear Regression model will suffice for this use case.

## Prediction

I visually show the accuracy of the model by taking 8 test datasets and seeding the `predict_series` function with the first observations of `boiler_temp_c` and `grouphead_temp_c`, producing a series of equal size to the test series.

![Predicted vs Real](figures/glm_pred_vs_real.png)
