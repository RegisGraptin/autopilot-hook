
# Autopilot Hook: ML-Driven Volatility Management for Uniswap V4
> Developed as part of the Uniswap Hook Incubator, powered by Atrium.


<div align="center">
    <img src="logo.jpeg" alt="Logo" width="320" height="320" />
</div>


**Autopilot Hook** is a Uniswap V4 hook designed to dynamically adjust fees based on market volatility using Machine Learning. By forecasting periods of high volatility, the system helps protect liquidity providers from impermanent loss. During low volatility, it lowers fees to incentivize trading activity.

## Our Approach

To forecast data volatility, we base our approach on price volatility. First, we define a window period used to measure volatility. Within this period, we calculate the absolute tick difference, which represents the price variation. Using these differences, we compute the standard deviation, a statistical measure of volatility for the given period.

For this study, we use price data sampled at 15-minute intervals, with a window composed of 20 ticks (equivalent to 5 hours). These parameters are adjustable and can significantly influence the results. For more volatile assets, it may be beneficial to refine these settings and analyze their impact further.

## Key Technologies

To implement this use case, we leverage the following technologies:

- **Uniswap V4**: Enables dynamic fee adjustments for liquidity providers by allowing hooks to modify the protocol’s behavior.
- **Brevis** (ZK Coprocessor): Computing the standard deviation on-chain can be gas-intensive. Instead, Brevis allows us to **process tick variations off-chain** securely and submit verified results on-chain using zero-knowledge proofs, ensuring trust in the computation.
- **Arbitrum Stylus**: Machine learning computations, such as forecasting volatility, are resource-intensive. Arbitrum Stylus **unlocks on-chain machine learning** capabilities by optimizing complex computations.





# Implementation: Building our Autopilot hook

## Data collection

Our research focuses on the **ETH/USD price data** collected from **November 2023** to **November 2024**, sampled at **15-minute intervals**. The dataset provides a granular view of price fluctuations, forming the foundation of our volatility prediction model.

To forecast volatility, we rely on the **last 20 data points** (spanning 5 hours) to capture short-term market behavior. These intervals allow us to track tick variations effectively and compute statistical metrics like the standard deviation for our analysis.


## Machine Learning model

For on-chain deployment, we implemented a **Linear Regression** model as a proof of concept, chosen for its simplicity and computational efficiency. This model forecasts volatility based on historical price data.

To ensure compatibility with on-chain arithmetic, we **avoided floating-point operations by upscaling both the input data and the model coefficients**. These values are converted to integers (int256) for processing directly within the smart contract. While this approach introduces minor approximations, it maintains prediction accuracy while adhering to on-chain computation constraints.

Detailed implementation files are available in the `model` folder

## Arbitrum Stylus implementation

We leverage Arbitrum Stylus to deploy our trained Linear Regression model directly on-chain. Stylus enables efficient and optimized execution of computationally intensive tasks, such as machine learning predictions, within smart contracts.

Our Stylus smart contract includes a function named `forecast_volatility`, which predicts the next volatility value based on historical data. Since at least 20 data points are required for predictions, the model **does not forecast during the first 20 calls**. Instead, it returns the current volatility. After the initial 20 calls, the model begins providing forecasts for the next volatility value.

The full implementation of this contract is located in the `stylus` folder.


## Brevis implementation

To reduce gas costs associated with on-chain data computation, we use Brevis, a tool that allows us to compute the necessary data off-chain. To do this, we created a **Brevis circuit** (located in `brevis/prover/circuits/circuit.go`) that **aggregates swap events and extracts tick price data**. From this data, we compute the standard deviation over our defined window period.

Once the circuit is built, we provide a Node.js server (see `brevis/server/api/index.ts`) that fetches swap events through an RPC server. This server also generates zero-knowledge proofs to ensure trust in the computation and can be used to update the volatility parameter on-chain.

We integrate this functionality into our smart contract by adding a callback to Brevis, allowing the contract to update the volatility value based on the off-chain computation.


## Hook Implementation

Now, in our hook, we are putting together all the different piece. As we are using dynamic fee, we need to check at the initialization, that the hook is intent to be dynamic. Then, we have a callback function for Brevis, allowing to store and update the volatility value. Finally, before each swap, we are checking the current volatility and the forecast one, and decide if we need to update or not the fee dynamically.


In the hook implementation, we integrate all the components necessary for dynamic fee adjustments.

- Initialization: First, we check that the hook is intended to support dynamic fees.
- Brevis Callback: We add a callback function for Brevis to store and update the volatility value in the smart contract.
- Dynamic Fee Adjustment: Before each swap, we verify that we are not entering a high volatility period by checking the forecasted volatility. If necessary, we adjust the dynamic fee to mitigate risk and protect liquidity providers.
This structured approach ensures that the dynamic fee adjustments are based on real-time volatility data, optimizing liquidity provider protection and trading incentives.



