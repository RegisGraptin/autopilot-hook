package circuits

import (
	"github.com/brevis-network/brevis-sdk/sdk"
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/common/hexutil"
)

// Add the possibility to adapt depending on the pool assets
type AppCircuit struct {
	PoolId sdk.Bytes32
}

var _ sdk.AppCircuit = &AppCircuit{}

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

	tickVariation := sdk.Map(receipts, func(cur sdk.Receipt) sdk.Uint248 {
		return api.ToUint248(cur.Fields[6].Value)
	})

	blockNums := sdk.Map(receipts, func(cur sdk.Receipt) sdk.Uint248 { return api.ToUint248(cur.BlockNum) })

	existing := sdk.Map(receipts, func(cur sdk.Receipt) sdk.Uint248 {
		return api.ToUint248(sdk.ConstUint64(1))
	})

	// Find out the minimum block number. This enables us to find out over what range
	// the user has a specific trading volume
	minBlockNum := sdk.Min(blockNums)

	sdk.Mean()

	// Sum up the volume of each trade
	sumNumberOfEvents := sdk.Sum(existing)

	api.OutputUint(248, sumNumberOfEvents)
	api.OutputUint(64, minBlockNum)

	return nil
}
