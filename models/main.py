from math import ceil
from numpy import diff
from pandas import DataFrame
from sklearn.metrics import mean_squared_error, r2_score
from visualisation import compare_plot
from thermal_loss.data_processing import group_measurements, resample_data, get_largest_timediff, split_x_y
from thermal_loss.model_glm import train_model, test_model, predict_series
from thermal_loss.dataset import read_thermal_loss_data

thermal_loss_data = read_thermal_loss_data()

groups = group_measurements(thermal_loss_data)

# for resample_time in ("min", "10S", "25S", "1T", "5T", "7.5T", "10T"):
#     groups = [resample_data(group, f"{ceil(get_largest_timediff(group).total_seconds())}S" if resample_time == "min" else resample_time) for group in groups]

#     X_train, y_train = prepare(groups[: int(len(groups) / 2)])
#     X_test, y_test = prepare(groups[int(len(groups) / 2) : len(groups)])

#     model = train_model(X_train, y_train)

#     y_hat = test_model(model, X_test)

#     mse = mean_squared_error(y_test, y_hat)
#     r2 = r2_score(y_test, y_hat)

#     print(f"{resample_time}. Mean squared: {y_train.mean() ** 2} MSE: {mse}, R2: {r2}")

resample_time = "1T"

groups = [
    resample_data(
        group,
        f"{ceil(get_largest_timediff(group).total_seconds())}S"
        if resample_time == "min"
        else resample_time,
    )
    for group in groups
]

train_groups = groups[: int(len(groups) / 2)]
test_groups = groups[int(len(groups) / 2) : len(groups)]

X_boiler_train, y_boiler_train = split_x_y(train_groups, X_col=["boiler_temp_c", "grouphead_temp_c"], y_col="boiler_temp_c")
X_grouphead_train, y_grouphead_train = split_x_y(train_groups, X_col=["boiler_temp_c", "grouphead_temp_c"], y_col="grouphead_temp_c")

X_test, y_test = split_x_y(test_groups, X_col=["boiler_temp_c", "grouphead_temp_c"], y_col="boiler_temp_c")

boiler_model = train_model(X_boiler_train, y_boiler_train)
grouphead_model = train_model(X_grouphead_train, y_grouphead_train)

test_group = test_groups[0]
test_group_inc = get_largest_timediff(test_group).total_seconds()

df = predict_series(
    boiler_model,
    grouphead_model,
    initial = (test_group.iloc[0]["boiler_temp_c"], test_group.iloc[0]["grouphead_temp_c"]),
    count = test_group.shape[0],
    time_increment=test_group_inc*1000
)

df["boiler_temp_c_real"] = test_group["boiler_temp_c"]

compare_plot(df, "Test", "time", ["boiler_temp_c", "boiler_temp_c_real"])

# y_hat = test_model(model, X_test)

# groups_with_yhat = []

# for group in test_groups:
#     group["boiler_temp_c_change_pred"] = [
#         model.predict([[row["boiler_temp_c"], row["grouphead_temp_c"]]])[0][0]
#         for _, row in group[["boiler_temp_c", "grouphead_temp_c"]].iterrows()
#     ]

#     boiler_temp_c_change_actual = diff(group["boiler_temp_c"].values).reshape(-1, 1)

#     group["boiler_temp_c_pred"] = [
#         row["boiler_temp_c"] + row["boiler_temp_c_change_pred"]
#         for _, row in group[["boiler_temp_c", "boiler_temp_c_change_pred"]].iterrows()
#     ]

#     groups_with_yhat.append(group)

# graph_predicted(groups_with_yhat, "Groups with yhat")
