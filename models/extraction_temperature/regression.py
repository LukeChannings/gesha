import json
from numpy import logspace
from sklearn.model_selection import GridSearchCV, train_test_split
from sklearn.linear_model import LinearRegression
from sklearn.metrics import mean_squared_error, r2_score
from skl2onnx import to_onnx
from sklearn.svm import SVR


def train_linear_regression(X, y):
    X_train, X_test, y_train, y_test = train_test_split(
        X, y, test_size=0.2, random_state=0
    )

    model = LinearRegression()

    # I don't need to scale the data because all the features are on the same scale
    model.fit(X_train, y_train)

    y_pred = model.predict(X_test)

    mse = mean_squared_error(y_test, y_pred)
    r2 = r2_score(y_test, y_pred)

    print(f"Mean Squared Error: {mse}")
    print(f"R^2 Score: {r2}")

    print(f"Model Coefficients: {model.coef_}")
    print(f"Model Intercept: {model.intercept_}")

    return model

def find_svm_hyperparameters(X, y):
    grid = GridSearchCV(
        SVR(kernel="rbf", gamma=0.1),
        param_grid={"C": [1e0, 1e1, 1e2, 1e3], "gamma": logspace(-2, 2, 5)},
    )

    grid.fit(X, y)

    print(f"Best parameters: {grid.best_params_}")
    print(f"Best cross-validation score: {grid.best_score_}")

def train_svm(X, y):
    from sklearn.model_selection import GridSearchCV, StratifiedShuffleSplit
    from sklearn.svm import SVR

    X_train, X_test, y_train, y_test = train_test_split(
        X, y, test_size=0.2, random_state=0
    )

    model = SVR(kernel="rbf", gamma=1.0, C=1000)
    model.fit(X_train, y_train)

    y_pred = model.predict(X_test)

    mse = mean_squared_error(y_test, y_pred)
    r2 = r2_score(y_test, y_pred)

    print(f"Mean Squared Error: {mse}")
    print(f"R^2 Score: {r2}")

    return model
