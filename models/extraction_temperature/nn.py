from keras.models import Sequential
from keras.layers import Dense, LSTM
from pandas import DataFrame
from sklearn.metrics import mean_squared_error, r2_score
from sklearn.model_selection import train_test_split
import tf2onnx
import tensorflow as tf
import numpy as np
import tract

from util import MODELS_PATH

ONNX_OUTPUT_PATH = str(MODELS_PATH / "extraction_temperature/output/extraction_temperature_2.onnx")

def train_neural_network(df: DataFrame, model_output_path = str(MODELS_PATH / "extraction_temperature/output/extraction_temperature_2.onnx")):
    X, y = df[["grouphead_temp_c", "boiler_temp_c"]], df["thermofilter_temp_c"]

    X_train, X_test, y_train, y_test = train_test_split(
        X, y, test_size=0.2, random_state=0
    )

    X_test = np.array(X_test).astype(np.float32)
    X_train = np.array(X_train).astype(np.float32)

    model = Sequential([
        Dense(2**6, activation='tanh', input_dim=2, name='input'),
        Dense(2**3, activation='tanh', name='hidden-1'),
        Dense(2**2, activation='tanh', name='hidden-2'),
        Dense(1, activation='linear', name="output"),
    ])

    model.compile(loss="mse", optimizer="rmsprop", metrics=["accuracy"])

    model.fit(X_train, y_train, epochs=500, batch_size=16)

    y_pred = model.predict(X_test)

    mse = mean_squared_error(y_test, y_pred)
    r2 = r2_score(y_test, y_pred)

    print(f"Mean Squared Error: {mse}")
    print(f"R^2 Score: {r2}")

    spec = (tf.TensorSpec((None, 2), tf.float32, name="input"),)

    # # Save the model in ONNX format to pass to tract
    tf2onnx.convert.from_keras(
        model,
        output_path=model_output_path,
        input_signature=spec
    )

    # Run the model in tract and check output against TensorFlow
    tract_model = tract.onnx().model_for_path(model_output_path)
    tract_model.set_output_fact(0, None)
    tract_output = tract_model.into_optimized().into_runnable().run([X_test])[0].to_numpy()
    assert(np.allclose(y_pred, tract_output))

    return model

def train_lstm_neural_network(df: DataFrame, model_output_path = str(MODELS_PATH / "extraction_temperature/output/extraction_temperature_lstm.onnx")):

    X, y = df[["grouphead_temp_c", "boiler_temp_c"]], df["thermofilter_temp_c"]

    X = X.iloc[:-(len(X) % 20)]
    X = X.to_numpy().reshape(-1, 20, 2).astype(np.float32)
    y = y[::20][:-1]

    X_train, X_test, y_train, y_test = train_test_split(
        X, y, test_size=0.2, random_state=0
    )

    model = Sequential([
        LSTM(2 ** 12,input_shape=(20,2), return_sequences=False, name="input"),
        Dense(1, activation='linear', name="output"),
    ])

    model.compile(loss="mse", optimizer="rmsprop")

    model.fit(X_train, y_train, epochs=500, batch_size=16)

    y_pred = model.predict(X_test)

    mse = mean_squared_error(y_test, y_pred)
    r2 = r2_score(y_test, y_pred)

    print(f"Mean Squared Error: {mse}")
    print(f"R^2 Score: {r2}")

    model.save(model_output_path + ".tf")

    # spec = (tf.TensorSpec((None, 10, 2), tf.float32, name="input"),)

    # # # Save the model in ONNX format to pass to tract
    # tf2onnx.convert.from_keras(
    #     model,
    #     output_path=model_output_path,
    #     input_signature=spec
    # )

    # # Run the model in tract and check output against TensorFlow
    # tract_model = tract.onnx().model_for_path(model_output_path)
    # tract_model.set_output_fact(0, None)
    # tract_output = tract_model.into_optimized().into_runnable().run([X_test])[0].to_numpy()
    # assert(np.allclose(y_pred, tract_output))

    # return model

def test_model(df: DataFrame, model_path: str):
    X, y = df[["grouphead_temp_c", "boiler_temp_c"]], df["thermofilter_temp_c"]

    X_train, X_test, y_train, y_test = train_test_split(
        X, y, test_size=0.2, random_state=0
    )

    X_test = np.array(X_test).astype(np.float32)

    tract_model = tract.onnx().model_for_path(ONNX_OUTPUT_PATH)
    tract_model.set_output_fact(0, None)
    model = tract_model.into_optimized().into_runnable()

    y_pred = model.run([X_test])[0].to_numpy()

    mse = mean_squared_error(y_test, y_pred)
    r2 = r2_score(y_test, y_pred)

    print(f"Mean Squared Error: {mse}")
    print(f"R^2 Score: {r2}")
