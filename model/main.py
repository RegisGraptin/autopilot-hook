import os 

import pandas as pd
import numpy as np
import matplotlib.pyplot as plt
import seaborn as sns

from sklearn.model_selection import train_test_split
from sklearn.linear_model import LinearRegression


def price_to_tick(price):
    return np.log(price) / np.log(1.0001)

def plot_data(df, column="tick_variation"):
    plt.figure(figsize=(12, 6))
    sns.lineplot(x='datetime', y=column, data=df, marker="o", label="Log Returns")
    plt.title("Log Returns over Time")
    plt.xlabel("Date")
    plt.ylabel("Log Return")
    plt.grid(True)
    plt.legend()
    plt.show()


def convert_date(df):
    df['date'] = pd.to_datetime(df['date'].astype(str), format='%Y%m%d')
    df['datetime'] = pd.to_datetime(df['date'].astype(str) + ' ' + df['time'])
    

def prepare_data_set():
    df = pd.read_csv("./data/ETHUSD_15_2023-11-29_2024-11-28.csv")

    df["tick"] = df["close"].apply(price_to_tick)
    df["tick"] = df["tick"].round().astype(int)

    df["tick_variation"] = np.log(df["tick"] / df["tick"].shift(1))

    # Convert date and time to datetime
    convert_date(df)
    # plot_data(df)

    # Drop first data as no tick variation data
    df = df.drop(index=0)

    df.to_csv("data.csv")



def create_training_data(df, column="rolling_std"):

    # Number of time steps to use for prediction (window size)
    window_size = 20

    # Create the dataset using a sliding window
    X = []
    y = []

    # Loop to create input (X) and output (y) pairs
    for i in range(len(df) - window_size):
        X.append(df[column].iloc[i:i + window_size].values)  # 20 tick variations
        y.append(df[column].iloc[i + window_size])  # 21st tick variation

    # Convert X and y to numpy arrays for model input
    X = np.array(X)
    y = np.array(y)

    # Check shapes of the resulting dataset
    print("Shape of X:", X.shape)  # (number of samples, 20)
    print("Shape of y:", y.shape)  # (number of samples,)

    return X, y

if __name__ == "__main__":
    if not os.path.exists("data.csv"):
        prepare_data_set()
    
    df = pd.read_csv("data.csv")

    # Scaled due to integer constraint in smart contract
    df['tick_variation'] = df['tick_variation'] * 10_000_000

    print(df.head())

    # Compute the absolute value of the tick variation to focus on volatility (magnitude)
    df['abs_tick_variation'] = df['tick_variation'].abs()

    window_size = 20

    # Compute the rolling mean and standard deviation
    df['rolling_mean'] = df['abs_tick_variation'].rolling(window=window_size).mean()
    df['rolling_std'] = df['abs_tick_variation'].rolling(window=window_size).std()

    # Drop rows where rolling calculations could not be made (first 'window_size' rows)
    df.dropna(inplace=True)


    print(df.head())

    # Create a target variable: predict the next standard deviation
    df['target'] = df['abs_tick_variation'].shift(-1)  # This shifts the 'tick_variation' column by 1 row (predict next)


    # print(df['target'])
    # print(df['rolling_std'])


    X, y = create_training_data(df)

    # Split the data into training and testing sets
    X_train, X_test, y_train, y_test = train_test_split(X, y, test_size=0.2, random_state=42)

    # print(df['rolling_std'].iloc[0:500])

    # plt.figure(figsize=(12, 6))
    # sns.lineplot(df['rolling_std'].iloc[0:500])
    # plt.grid(True)
    # plt.legend()
    # plt.show()


    # Train a simple Linear Regression model
    model = LinearRegression()
    model.fit(X_train, y_train)

    # Evaluate the model
    print("Model R^2 score on test data:", model.score(X_test, y_test))

    print(model.get_params())
    print(model.coef_)


