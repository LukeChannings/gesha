from keras.models import Sequential, Model
from keras.layers import Dense, Input
from keras.callbacks import EarlyStopping
from pandas import DataFrame
from sklearn.metrics import mean_squared_error, r2_score
from sklearn.model_selection import train_test_split
import tf2onnx
import tensorflow as tf
import numpy as np
import pandas as pd
import tract
from predictive.dataset import get_correlation_optimised_measurement_data, group_by_contiguous_measurements, upscale_data

from util import MODELS_PATH

ONNX_OUTPUT_PATH = str(MODELS_PATH / "predictive/output/predictive.onnx")

def train_simple_neural_network(df: DataFrame, model_path: str):

    X = (
        df[["grouphead_temp_c", "boiler_temp_c", "rolling_heat_level"]]
        .to_numpy()
        .astype(np.float32)
    )
    y = df["future_temp_diff"].to_numpy().astype(np.float32)

    X_train, X_test, y_train, y_test = train_test_split(
        X, y, test_size=0.25, random_state=0
    )

    # Define the model
    model = Sequential([
        Dense(64, input_dim=3, dtype=tf.float32, activation='relu', name="hidden_1"),
        Dense(32, dtype=tf.float32, activation='relu', name="hidden_2"),
        Dense(1, dtype=tf.float32, name="output"),
    ])

    model.compile(loss="mse", optimizer="adam")

    early_stop = EarlyStopping(monitor='loss', patience=200, verbose=1, restore_best_weights=True)

    model.fit(X_train, y_train, epochs=3000, batch_size=32, callbacks=[early_stop])

    y_pred = model.predict(X_test)

    mse = mean_squared_error(y_test, y_pred)
    r2 = r2_score(y_test, y_pred)

    print(f"Mean Squared Error: {mse}")
    print(f"R^2 Score: {r2}")

    spec = (tf.TensorSpec((None, 3), tf.float32, name="input"),)

    # # Save the model in ONNX format to pass to tract
    tf2onnx.convert.from_keras(
        model, output_path=model_path, input_signature=spec
    )

    # Run the model in tract and check output against TensorFlow
    tract_model = tract.onnx().model_for_path(model_path)
    tract_model.set_output_fact(0, None)
    tract_output = (
        tract_model.into_optimized().into_runnable().run([X_test])[0].to_numpy()
    )
    assert np.allclose(y_pred, tract_output, atol=0.01, equal_nan=True)

    return model


def train_simple_nn_1(df: DataFrame):
    groups = group_by_contiguous_measurements(df)
    model_path = str(MODELS_PATH / "predictive/output/predictive-10-09-2023.onnx")

    def process_data(df: DataFrame):
        df = upscale_data(df, period="100ms")
        df["rolling_heat_level"] = df["heat_level"].rolling(window=260).sum().fillna(0)

        df["future_temp_diff"] = df["boiler_temp_c"].shift(-2160)

        df.dropna(inplace=True)

        return df

    df = pd.concat([process_data(g) for g in groups])

    X = (
        df[["grouphead_temp_c", "boiler_temp_c", "rolling_heat_level"]]
        .to_numpy()
        .astype(np.float32)
    )

    y = df["future_temp_diff"].to_numpy().astype(np.float32)

    X_train, X_test, y_train, y_test = train_test_split(
        X, y, test_size=0.33, random_state=0
    )

    model = Sequential(
        [
            Dense(64, activation="tanh", input_dim=3, dtype=tf.float32, name="hidden1"),
            Dense(8, activation="tanh", dtype=tf.float32, name="hidden2"),
            Dense(1, activation="linear", dtype=tf.float32, name="output"),
        ]
    )

    model.compile(loss="mse", optimizer="adam")

    model.fit(X_train, y_train, epochs=50, batch_size=128)

    y_pred = model.predict(X_test)

    mse = mean_squared_error(y_test, y_pred)
    r2 = r2_score(y_test, y_pred)

    print(f"Mean Squared Error: {mse}")
    print(f"R^2 Score: {r2}")

    spec = (tf.TensorSpec((None, 3), tf.float32, name="input"),)

    # # Save the model in ONNX format to pass to tract
    tf2onnx.convert.from_keras(
        model, output_path=model_path, input_signature=spec
    )


    return model

def evaluate_simple_nn_1(df: DataFrame):
    groups = group_by_contiguous_measurements(df)
    model_path = str(MODELS_PATH / "predictive/output/predictive-10-09-2023.onnx")

    def process_data(df: DataFrame):
        df = upscale_data(df, period="100ms")
        df["rolling_heat_level"] = df["heat_level"].rolling(window=260).sum().fillna(0)

        df["future_temp_diff"] = df["boiler_temp_c"].shift(-2160)

        df.dropna(inplace=True)

        return df

    df = pd.concat([process_data(g) for g in groups])

    X = (
        df[["grouphead_temp_c", "boiler_temp_c", "rolling_heat_level"]]
        .to_numpy()
        .astype(np.float32)
    )

    y = df["future_temp_diff"].to_numpy().astype(np.float32)

    X_train, X_test, y_train, y_test = train_test_split(
        X, y, test_size=0.33, random_state=0
    )
    # Run the model in tract and check output against TensorFlow
    tract_model = tract.onnx().model_for_path(model_path)
    tract_model.set_output_fact(0, None)
    y_pred = (
        tract_model.into_optimized().into_runnable().run([X_test])[0].to_numpy()
    )

    mse = mean_squared_error(y_test, y_pred)
    r2 = r2_score(y_test, y_pred)

    print(f"Mean Squared Error: {mse}")
    print(f"R^2 Score: {r2}")

def train_heat_sum_model(df: DataFrame):
    X = (
        df[[
            "temp_initial",
            "temp_initial_grouphead",
            "temp_diff",
        ]]
        .to_numpy()
        .astype(np.float32)
    )
    y = df["heat_level_sum"].to_numpy().astype(np.float32)

    X_train, X_test, y_train, y_test = train_test_split(
        X, y, test_size=0.1, random_state=0
    )

    model = Sequential(
        [
            Dense(12, activation="tanh", input_dim=3, dtype=tf.float32, name="input"),
            Dense(6, activation="tanh", dtype=tf.float32, name="hidden_1"),
            Dense(3, activation="tanh", dtype=tf.float32, name="hidden_2"),
            Dense(1, activation="linear", dtype=tf.float32, name="output"),
        ]
    )

    model.compile(loss="mse", optimizer="adam")

    model.fit(X_train, y_train, epochs=10_000, batch_size=8)

    y_pred = model.predict(X_test)

    mse = mean_squared_error(y_test, y_pred)
    r2 = r2_score(y_test, y_pred)

    print(f"Mean Squared Error: {mse}")
    print(f"R^2 Score: {r2}")

    spec = (tf.TensorSpec((None, 3), tf.float32, name="input"),)

    model_output_path = str(MODELS_PATH / "temp_to_heat_sum.onnx")

    # # Save the model in ONNX format to pass to tract
    tf2onnx.convert.from_keras(
        model, output_path=model_output_path, input_signature=spec
    )

    # Run the model in tract and check output against TensorFlow
    tract_model = tract.onnx().model_for_path(model_output_path)
    tract_model.set_output_fact(0, None)
    tract_output = (
        tract_model.into_optimized().into_runnable().run([X_test])[0].to_numpy()
    )
    assert np.allclose(y_pred, tract_output, atol=0.01, equal_nan=True)

    return model
# For REPL:

from util import MODELS_PATH
import tract
import numpy as np

ONNX_OUTPUT_PATH = str(MODELS_PATH / "predictive/output/subset_model.onnx")
tract_model = tract.onnx().model_for_path(ONNX_OUTPUT_PATH)
tract_model.set_output_fact(0, None)
model = tract_model.into_optimized().into_runnable()
model.run([np.array([[80.0, 90.0, 245]]).astype(np.float32)])[0].to_numpy().item()
