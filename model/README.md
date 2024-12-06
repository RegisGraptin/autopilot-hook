
# ForcastML - Market Volatility Forecast

Our goal is to develop a simple machine learning model to predict **market volatility** based on historical price behavior.
This model will initially serve as a proof of concept for forecasting price volatility and will later be integrated into a smart contract using Stylus from Arbitrum to enable on-chain execution.
By leveraging historical data, we aim to build a robust system that dynamically adjusts fees based on market conditions, helping to protect liquidity providers (LPs) from the risks associated with sudden price volatility in the cryptocurrency market.

## Data Collection & Scope

Our research focuses on the **ETH/USD price data** from **November 2023** to **November 2024**, sampled at **15-minute intervals**.
This dataset provides a granular view of price fluctuations, which will serve as the foundation for our volatility prediction model.

### Limitations & Considerations:

- We are using only one year of data, which introduces a potential bias, as cryptocurrency markets often follow longer cycles (e.g., 4-year cycles).
- The crypto market is inherently more volatile compared to traditional financial markets, which may impact the accuracy and stability of our model.
- As the cryptocurrency market matures, past volatility patterns may become less relevant. However, analyzing historical events like flash crashes can still provide valuable insights and help validate our approach in real-world scenarios.


## Model Design & Implementation

Our approach uses historical price data to predict price volatility. For that, we are relying on the last **20 data points** (15-minute intervals) to capture short-term market behavior.

### Data Transformation:

We start by converting the raw price data into **tick values** and calculate the **tick differences** between each 15-minute interval to measure volatility. To measure volatility, we are going to focus our approach on price variation over a window of 20 intervals.

- For each time step t, we group the previous 20 tick variations (t-20 to t) to compute the standard deviation.
- The standard deviation serves as our primary indicator of price variation, which we aim to predict.

Given the need for on-chain deployment, our initial implementation uses a Linear Regression model as a proof of concept due to its simplicity and efficiency.
To avoid handling floating-point numbers on-chain, we decided to upscale both the input data and the model coefficients to be able to use int256 directly in the smart contract. While this introduces minor approximations, it ensures compatibility with on-chain arithmetic without compromising too much prediction accuracy.

## Environement

Some command to create locally your environment:

```bash
uv venv
source .venv/bin/activate
```


Manage new dependencies
```bash
uv pip freeze | uv pip compile - -o requirements.txt
```

## Run script

```bash
source .venv/bin/activate
python main.py
```