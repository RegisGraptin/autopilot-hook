package circuits

import (
	"github.com/brevis-network/brevis-sdk/sdk"
	"github.com/ethereum/go-ethereum/common/hexutil"
)

type AppCircuit struct {
	// You can define your own custom circuit inputs here, but note that they cannot
	// have the `gnark:",public"` tag.
	UserAddr sdk.Uint248
}

// Your guest circuit must implement the sdk.AppCircuit interface
var _ sdk.AppCircuit = &AppCircuit{}

// sdk.ParseXXX APIs are used to convert Go/EVM data types into circuit types.
// Note that you can only use these outside of circuit (making constant circuit
// variables)

// event BuyBottle (address indexed user, uint256 tokenId);
// Obtain from the events list
// https://sepolia.etherscan.io/address/0x5d577a3e774622f49f333499534be55a2cf5d4f6#events

var EventBuyBottle = sdk.ParseEventID(hexutil.MustDecode("0x822cd8cda2d2b3feb2339ab2e79a3b336b792f0a912867901e963e928afd3be8")) // FIXME:

var WineAddr = sdk.ConstUint248("0x5d577A3e774622f49f333499534be55a2Cf5d4F6")

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

	receipts := sdk.NewDataStream(api, in.Receipts)

	// Run for each receipt
	sdk.AssertEach(receipts, func(l sdk.Receipt) sdk.Uint248 {

		// Check the event is related to the users
		assertionPassed := u248.And(
			// Check the right event
			u248.IsEqual(l.Fields[0].EventID, EventBuyBottle),

			// Check the targeted user
			u248.IsEqual(api.ToUint248(l.Fields[1].Value), c.UserAddr),
		)

		return assertionPassed
	})

	blockNums := sdk.Map(receipts, func(cur sdk.Receipt) sdk.Uint248 { return api.ToUint248(cur.BlockNum) })

	existing := sdk.Map(receipts, func(cur sdk.Receipt) sdk.Uint248 {
		return api.ToUint248(sdk.ConstUint64(1))
	})

	// Find out the minimum block number. This enables us to find out over what range
	// the user has a specific trading volume
	minBlockNum := sdk.Min(blockNums)

	// Sum up the volume of each trade
	sumNumberOfEvents := sdk.Sum(existing)

	api.OutputUint(248, sumNumberOfEvents)
	api.OutputUint(64, minBlockNum)
	api.OutputAddress(c.UserAddr)

	return nil
}
