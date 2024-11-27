
# Autopilot Hook
> Built during the Uniswap Hook Incubator by Atrium

We develop **Autopilot Hook** a Uniswap hook design to dynamically adjust fees based on market volatility using a Machine Learning approach. 

In our approach, we are using internal data to state about the current market state and also Brevis, allowing us to do computation off-chain to feed in our machine learning model.

At each block, the model try to forcast the market condition, and based on his prediction and analyse will update the fees accordingy.

The main idea behind it is to increase the fees during high volatility to protect liquidity providers and mitigate impermanent loss, while lowering fees during low volatility to encourage trading.


## Data collection


Regarding the data collection


- Total liquidity (TVL - Total Value Locked).
- Volume traded in the pool.
- Number of swaps per time period.
- Fee tier currently applied.
- Token reserves (reserve0, reserve1).
- Liquidity position changes (additions/removals).
- Number of liquidity providers.



## Sponsors

- Uniswap: Target dynamic fees based on volatility
- Arbitrum: ML model off-chain to predict future volatility and embed this model in Stylus for efficient and low-cost execution.
- Brevis: Periodically compute market volatility indexes - Feed that to a dynamic fee hook to determine the fee percentage

