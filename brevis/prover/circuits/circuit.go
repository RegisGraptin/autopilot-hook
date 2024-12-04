package circuits

import (
	"fmt"

	"github.com/brevis-network/brevis-sdk/sdk"
	"github.com/consensys/gnark/frontend"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/hexutil"
)

// Add the possibility to adapt depending on the pool assets
type AppCircuit struct {
	PoolId sdk.Bytes32
}

var _ sdk.AppCircuit = &AppCircuit{}

// Define a custom accumulator struct
type TickAccumulator struct {
	PreviousTick sdk.Uint248 // Store the previous tick value
	Sum          sdk.Uint248 // Sum of tick variations
	Count        sdk.Uint248 // Count of tick variations
	SumOfSquares sdk.Uint248 // Sum of squared differences for standard deviation
}

func (acc TickAccumulator) FromValues(values ...frontend.Variable) sdk.CircuitVariable {
	if len(values) != 4 {
		// In case of an error, you might want to return an empty accumulator or handle it differently
		panic(fmt.Sprintf("expected 4 values, got %d", len(values))) // Or use error handling as per your design
	}

	acc.PreviousTick = values[0].(sdk.Uint248)
	acc.Sum = values[1].(sdk.Uint248)
	acc.Count = values[2].(sdk.Uint248)
	acc.SumOfSquares = values[3].(sdk.Uint248)

	return nil
}

// Implement the NumVars method to satisfy sdk.CircuitVariable
func (acc TickAccumulator) NumVars() uint32 {
	return 4 // The number of variables in the TickAccumulator (PreviousTick, Sum, Count, SumOfSquares)
}

// Implement the String method to satisfy sdk.CircuitVariable
func (acc TickAccumulator) String() string {
	return fmt.Sprintf("TickAccumulator{PreviousTick: %s, Sum: %s, Count: %s, SumOfSquares: %s}",
		acc.PreviousTick.String(), acc.Sum.String(), acc.Count.String(), acc.SumOfSquares.String())
}

// Implement the Values method to satisfy sdk.CircuitVariable
func (acc TickAccumulator) Values() []frontend.Variable {
	// Return a slice of frontend.Variable containing the values from the TickAccumulator fields
	return []frontend.Variable{
		acc.PreviousTick, // PreviousTick as a frontend.Variable
		acc.Sum,          // Sum as a frontend.Variable
		acc.Count,        // Count as a frontend.Variable
		acc.SumOfSquares, // SumOfSquares as a frontend.Variable
	}
}

// /// @notice Emitted for swaps between currency0 and currency1
// /// @param id The abi encoded hash of the pool key struct for the pool that was modified
// /// @param sender The address that initiated the swap call, and that received the callback
// /// @param amount0 The delta of the currency0 balance of the pool
// /// @param amount1 The delta of the currency1 balance of the pool
// /// @param sqrtPriceX96 The sqrt(price) of the pool after the swap, as a Q64.96
// /// @param liquidity The liquidity of the pool after the swap
// /// @param tick The log base 1.0001 of the price of the pool after the swap
// /// @param fee The swap fee in hundredths of a bip
// event Swap(
//     PoolId indexed id,
//     address indexed sender,
//     int128 amount0,
//     int128 amount1,
//     uint160 sqrtPriceX96,
//     uint128 liquidity,
//     int24 tick,
//     uint24 fee
// );

// Example event
// emit Swap(id: 0x9adbefe3f238ee484d264492f59ec821d3f0b665750140ee12c0c19f4f2af2b7, sender: PoolSwapTest: [0x2e234DAe75C793f67A35089C9d99245E1C58470b], amount0: -10000000000000 [-1e13], amount1: 9899997049800 [9.899e12], sqrtPriceX96: 79228146787477200279319694892 [7.922e28], liquidity: 100000000000000000000 [1e20], tick: -1, fee: 10000 [1e4])

// Swap & Transfer event
var EventIdSwap = sdk.ParseEventID(hexutil.MustDecode("0xc42079f94a6350d7e6235f29174924f928cc2ac818eb64fed8004e115fbcca67"))

var UniswapPoolAddress = sdk.ConstUint248(common.HexToAddress("0xEf1c6E67703c7BD7107eed8303Fbe6EC2554BF6B"))

func (c *AppCircuit) Allocate() (maxReceipts, maxSlots, maxTransactions int) {
	// Allocating regions for different source data. Here, we are allocating 5 data
	// slots for "receipt" data, and none for other data types. Please note that if
	// you allocate it this way and compile your circuit, the circuit structure will
	// always have 5 processing "chips" for receipts and none for others. It means
	// your compiled circuit will always be only able to process up to 5 receipts and
	// cannot process other types unless you change the allocations and recompile.
	return 32, 0, 0
}

func (c *AppCircuit) Define(api *sdk.CircuitAPI, in sdk.DataInput) error {
	u248 := api.Uint248
	bytes32 := api.Bytes32

	receipts := sdk.NewDataStream(api, in.Receipts)

	// Filter on the receipt
	sdk.AssertEach(receipts, func(l sdk.Receipt) sdk.Uint248 {

		// Check the event is related to the users
		assertionPassed := u248.And(
			// Check the right event
			u248.IsEqual(l.Fields[0].EventID, EventIdSwap),

			// Filter on the poolId
			bytes32.IsEqual(l.Fields[1].Value, c.PoolId),

			// Filter on the sender
			u248.IsEqual(api.ToUint248(l.Fields[2].Value), UniswapPoolAddress),
		)

		return assertionPassed
	})

	// Get only the tick values
	tickValues := sdk.Map(receipts, func(cur sdk.Receipt) sdk.Uint248 {
		return api.ToUint248(cur.Fields[3].Value)
	})

	//// Now compute the std from the tick variation

	// First pass: Compute the sum and count of tick variations
	tickStats := sdk.Reduce(tickValues, TickAccumulator{
		Sum:          sdk.Uint248{Val: 0},
		Count:        sdk.Uint248{Val: 0},
		PreviousTick: sdk.Uint248{Val: 0},
		SumOfSquares: sdk.Uint248{Val: 0}, // Used in the second pass
	}, func(acc TickAccumulator, curr sdk.Uint248) TickAccumulator {

		// We are skiping the first item
		hasCountValue := api.Uint248.IsGreaterThan(acc.Count, sdk.Uint248{Val: 0}) == sdk.Uint248{Val: 0}
		if hasCountValue {

			// Calculate the tick variation as the difference between current and previous tick
			// Do not forget to scale it for our ML model
			val := api.Uint248.Mul(curr, sdk.Uint248{Val: 10_000_000})
			tickVariation, _ := api.Uint248.Div(val, acc.PreviousTick)

			// Update the accumulator: sum and count
			acc.Sum = api.Uint248.Add(acc.Sum, tickVariation)
			acc.Count = api.Uint248.Add(acc.Count, sdk.Uint248{Val: 1})
		}

		acc.PreviousTick = curr

		return acc
	})

	// Compute the mean (sum / count)
	mean, _ := api.Uint248.Div(tickStats.Sum, tickStats.Count)

	// Second pass: Compute the sum of squared differences from the mean
	sumOfSquaredDifferences := sdk.Reduce(tickValues, TickAccumulator{
		SumOfSquares: sdk.Uint248{Val: 0},
		Count:        sdk.Uint248{Val: 0},
		PreviousTick: sdk.Uint248{Val: 0},
	}, func(acc TickAccumulator, curr sdk.Uint248) TickAccumulator {

		// We are skiping the first item
		hasCountValue := api.Uint248.IsGreaterThan(acc.Count, sdk.Uint248{Val: 0}) == sdk.Uint248{Val: 0}
		if hasCountValue {

			// Again compute tick variation
			val := api.Uint248.Mul(curr, sdk.Uint248{Val: 10_000_000})
			tickVariation, _ := api.Uint248.Div(val, acc.PreviousTick)

			squaredDifference := api.Uint248.Mul(api.Uint248.Sub(tickVariation, mean), api.Uint248.Sub(tickVariation, mean))
			acc.SumOfSquares = api.Uint248.Add(acc.SumOfSquares, squaredDifference)

			acc.PreviousTick = curr
			acc.Count = api.Uint248.Add(acc.Count, sdk.Uint248{Val: 1})

		}

		return acc
	})

	// Compute the standard deviation (sqrt(sumOfSquares / count))
	v, _ := api.Uint248.Div(sumOfSquaredDifferences.SumOfSquares, tickStats.Count)
	std := api.Uint248.Sqrt(v)

	// Output the standard deviation
	api.OutputUint(248, std)

	return nil
}
