
# Autopilot Hook
> Built during the Uniswap Hook Incubator by Atrium

We develop **Autopilot Hook** a Uniswap hook design to dynamically adjust fees based on market volatility using a Machine Learning approach. 

In our approach, we are using internal data to state about the current market state and also Brevis, allowing us to do computation off-chain to feed in our machine learning model.

At each block, the model try to forcast the market condition, and based on his prediction and analyse will update the fees accordingy.

The main idea behind it is to increase the fees during high volatility to protect liquidity providers and mitigate impermanent loss, while lowering fees during low volatility to encourage trading.


## Data collection

We defined volatility by the price impact, using the Moving Average Volatility. 
We will use a rolling windows, to determine the price volatility.


To reduce impermanent loss, we will focus on price movement.


Computing moving average on chain can be costly, as we need it to update it for each windows block. 
An alternative is to use Brevis, allowing us to compute long and short term moving average.

In that case, we can use this data to predict price volatility.






Regarding the data collection


- Pool prize (reserveB / reserveA)
- Liquidity reserveA, reserveB, totalLiquidity
- Large swaps
- price impact




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

