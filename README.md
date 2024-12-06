
# Autopilot Hook
> Built during the Uniswap Hook Incubator by Atrium

We develop **Autopilot Hook** a Uniswap hook design to dynamically adjust fees based on market volatility using a Machine Learning approach. 

In case of a forcast **high volatlity**, we want to be able to protect the liquidity providers and mitigate impermanent loss, while lowering fees during low volatility to encourage trading.

## Our Approach

To be able to forcast data volatility, we have decided to based our approach on price volatility. To do that, we are first defining a window period, that is going to be used to measure ou volatility. Then, on this period, we can compute the the absolute tick difference. Finally, based on this difference, we are computing the standard deviation of the given periode. 

For our study, we have decided to study a tick variation on each 15 minutes. And our window, will be compose of 20 ticks, meaning a window of 5 hours. Notice, that those parameter can be changed and will impact the market. Depending of the assets choosen, some more volatile than other, it could be wise to adjust and study the impact.

## Technologies

To implement this use case, we are leveraging **Uniswap V4** to be able to adapt/adjust the dynamic fee. As you may notice, to be able to use and compute the standard deviation, as it could be gas consuming of doing it on chain, we are leveraging **Brevis** a ZK-Coprossesor allowing us to compute the tick variation off-chain and feed it on-chain by having a proof allowing us to trust the computation. Finally, for our machine learning model, as the computation for the forcast can be complex, we are using **Arbitrum Stylus** allowing us to unlock Machine Learning process directly on-chain, by optimizing computation.


# Implementation

## Data collection

Our research focuses on the **ETH/USD price data** from **November 2023** to **November 2024**, sampled at **15-minute intervals**.
This dataset provides a granular view of price fluctuations, which will serve as the foundation for our volatility prediction model.

Our approach uses historical price data to predict price volatility. For that, we are reling on the last **20 data points** (15-minute intervals) to capture short-term market behavior.


## Machine Learning model

Given the need for on-chain deployment, our initial implementation uses a Linear Regression model as a proof of concept due to its simplicity and efficiency.
To avoid handling floating-point numbers on-chain, we decided to upscale both the input data and the model coefficients to be able to use int256 directly in the smart contract. While this introduces minor approximations, it ensures compatibility with on-chain arithmetic without compromising too much prediction accuracy.

All the detail of the implementation can be found in the `model` folder. 

## Arbitrum Stylus implementation

On the Arbiturm Stylus implementation, we have imported our trained model, and defined it directly in the smart contract. Our stylus smart contract have one function called `forcast_volatility` which, as the name suggest it, will predict the next volatility. Notice that for the first 20 called, the model will not forcast any point, as we need to have at least 20 data history to be able to predict the next one. Thus, for the 20th first call, we are returning the current volatility, and after that, we are forcast the new one.  

You can found the implementation of the contract in the `stylus` folder. 

## Brevis - How to fetch our data

As computing our data, can be gas consuming to do it on-chain, we are using Brevis, allowing us to compute it off-chain. For that, we have created a circuit (see `brevis/prover/circuits/circuit.go`) where we are aggregating all the swap events, to extract the tick price, and we are computing the standard deviation from our window period.

Once the circuit built, we also provide a node server, allowing us to fetch the different Swap events, using a RPC server, and allowing us to generate the proof. This implementation can be found in the `brevis/server/api/index.ts` file.

We also add in our smart contract the callback call for Brevis, which is going to update the volatility parameter.

## Hook Implementation

Now, in our hook, we are putting together all the different piece. As we are using dynamic fee, we need to check at the initialization, that the hook is intent to be dynamic. Then, we have a callback function for Brevis, allowing to store and update the volatility value. Finally, before each swap, we are checking the current volatility and the forcast one, and decide if we need to update or not the fee dynamically.

