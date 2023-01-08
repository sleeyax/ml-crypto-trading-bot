import pandas as pd
import numpy as np
import math

from sklearn.metrics import mean_squared_error, mean_absolute_error, explained_variance_score, r2_score
from sklearn.metrics import mean_poisson_deviance, mean_gamma_deviance, accuracy_score
from sklearn.preprocessing import MinMaxScaler

from tensorflow.python.keras.models import Sequential
from tensorflow.python.keras.layers import Dense, Dropout, LSTM
from tensorflow.python.keras.models import load_model as load_model_from_file
from keras.callbacks import EarlyStopping

scaler = MinMaxScaler(feature_range=(0, 1))

# Input variables to make predictions.
FEATURES = ["open_date"]

# Output variables to predict.
LABELS = ["close"]


def load_dataset(path: str):
    """
    Load a dataframe from the given dataset file.
    Only supports CSV files for now.
    """

    # Read dataframe form the given .csv file.
    df = pd.read_csv(path)

    # Add pandas datetime fields.
    # The dataset from Binance only contains numeric epoch timestamps, making it harder to work with later on.
    df[FEATURES[0]] = pd.to_datetime(df['open_time'], unit='ms')
    df['close_date'] = pd.to_datetime(df['close_time'], unit='ms')

    # Print some info about the resulting dataframe.
    start_date = df.iloc[0][0]
    end_date = df.iloc[-1][0]
    print('Total number of rows present in the dataset:', df.shape[0])
    print('Total number of columns present in the dataset:', df.shape[1])
    print(
        f"Starting date: {pd.to_datetime(start_date, unit='ms')} ({start_date})")
    print(f"Ending date:   {pd.to_datetime(end_date, unit='ms')} ({end_date})")

    return df


def get_prediction_dataframe(df: pd.DataFrame):
    """
    Returns the labels and features to use for training. 
    """

    # Only include the data we are interested in for training.
    df = df[FEATURES + LABELS]

    # Only include market data from 2021 to today.
    # BTC has changed too much in earlier years which affects prediction accuracy.
    df = df[df[FEATURES[0]] > '2021-01-01']

    print("Shape of prediction dataframe:", df.shape)

    return df


def normalize(df: pd.DataFrame):
    """
    Normalize by scaling the data.
    """

    # TODO: is deleting features really required for normalization?
    del df[FEATURES[0]]

    # BTC prices fluctuate a lot, so let's rescale it.
    df = scaler.fit_transform(np.array(df).reshape(-1, 1))

    print("Shape after normalization:", df.shape)

    return df


def unnormalize(data):
    """
    Undo normalization.
    """

    return scaler.inverse_transform(data)


def split_dataset(df: pd.DataFrame):
    """
    Split the provided frame into training and testing data.
    """

    # Split frame into 70% training and 30% tesing data.
    training_percentage = 0.70
    training_size = int(len(df) * training_percentage)
    train_data, test_data = df[0:training_size,
                               :], df[training_size:len(df), :1]

    print("Train data shape: ", train_data.shape)
    print("Test data shape: ", test_data.shape)

    return (train_data, test_data)


def create_dataset_matrix(dataset, time_step: int):
    """
    Convert an array of values into a dataset matrix.
    """
    dataX, dataY = [], []
    for i in range(len(dataset)-time_step-1):
        a = dataset[i:(i+time_step), 0]  # i=0, 0,1,2,3-----99   100
        dataX.append(a)
        dataY.append(dataset[i + time_step, 0])
    return np.array(dataX), np.array(dataY)


def normalize_lstm(arr: np.ndarray):
    """
    Reshape input to be `[samples, time steps, features]`, which is required for LSTM.
    """
    return arr.reshape(arr.shape[0], arr.shape[1], 1)


def create_model():
    model = Sequential()
    model.add(LSTM(20, input_shape=(None, 1), activation="tanh"))
    model.add(Dense(1))
    model.compile(loss="mean_absolute_error", optimizer="adam")
    return model

def load_model(path: str):
    return load_model_from_file(path)

def train_model(model: Sequential, X_train, y_train, X_test, y_test):
    model.fit(X_train, y_train, validation_data=(X_test, y_test), epochs=200, batch_size=32, verbose=1, callbacks=[
        EarlyStopping(monitor="val_loss", verbose=2, mode='min', patience=5)
    ])
    return model


def evaluate_performance(train_predict, test_predict, y_train, y_test):
    original_ytrain = unnormalize(y_train.reshape(-1, 1))
    original_ytest = unnormalize(y_test.reshape(-1, 1))

    # Evaluation metrices RMSE, MSE and MAE
    print("Train data RMSE: ", math.sqrt(
        mean_squared_error(original_ytrain, train_predict)))
    print("Train data MSE: ", mean_squared_error(
        original_ytrain, train_predict))
    print("Train data MAE: ", mean_absolute_error(
        original_ytrain, train_predict))
    print("-------------------------------------------------------------------------------------")
    print("Test data RMSE: ", math.sqrt(
        mean_squared_error(original_ytest, test_predict)))
    print("Test data MSE: ", mean_squared_error(original_ytest, test_predict))
    print("Test data MAE: ", mean_absolute_error(original_ytest, test_predict))

    # %% [markdown]
    # Variance Regression Score

    print("Train data explained variance regression score:",
          explained_variance_score(original_ytrain, train_predict))
    print("Test data explained variance regression score:",
          explained_variance_score(original_ytest, test_predict))

    # R square score for regression
    print("Train data R2 score:", r2_score(original_ytrain, train_predict))
    print("Test data R2 score:", r2_score(original_ytest, test_predict))

    # Regression Loss Mean Gamma deviance regression loss (MGD) and Mean Poisson deviance regression loss (MPD)
    print("Train data MGD: ", mean_gamma_deviance(
        original_ytrain, train_predict))
    print("Test data MGD: ", mean_gamma_deviance(original_ytest, test_predict))
    print("----------------------------------------------------------------------")
    print("Train data MPD: ", mean_poisson_deviance(
        original_ytrain, train_predict))
    print("Test data MPD: ", mean_poisson_deviance(original_ytest, test_predict))
