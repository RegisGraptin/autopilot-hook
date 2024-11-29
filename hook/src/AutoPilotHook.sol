// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import {BaseHook} from "v4-periphery/src/base/hooks/BaseHook.sol";
import {IPoolManager} from "v4-core/interfaces/IPoolManager.sol";
import {Hooks} from "v4-core/libraries/Hooks.sol";
import {PoolKey} from "v4-core/types/PoolKey.sol";
import {BalanceDelta} from "v4-core/types/BalanceDelta.sol";
import {LPFeeLibrary} from "v4-core/libraries/LPFeeLibrary.sol";
import {BeforeSwapDelta, BeforeSwapDeltaLibrary} from "v4-core/types/BeforeSwapDelta.sol";

import {BrevisApp} from "@brevis-network/contracts/sdk/apps/framework/BrevisApp.sol";

import {Ownable} from "@openzeppelin/contracts/access/Ownable.sol";

// Make sure to update the interface when Stylus Contract's Solidity ABI changes.
interface IDynamicMLFee { // FIXME:
    function getAmountInForExactOutput(uint256 amountOut, address input, address output, bool zeroForOne)
        external
        returns (uint256);

    function getAmountOutFromExactInput(uint256 amountIn, address input, address output, bool zeroForOne)
        external
        returns (uint256);
}

contract AutoPilotHook is BaseHook, BrevisApp, Ownable {
    using LPFeeLibrary for uint24;

    bytes32 public vkHash;

    IDynamicMLFee _dynamicFee;

    constructor(IPoolManager _poolManager, address brevisRequest, address stylusMLContract) 
        BaseHook(_poolManager)
        BrevisApp(brevisRequest)
        Ownable(msg.sender)
    {
        _dynamicFee = IDynamicMLFee(stylusMLContract);
    }

    // BrevisQuery contract will call our callback once Brevis backend submits the proof.
    function handleProofResult(  // solhint-disable-line private-vars-leading-underscore
        bytes32 _vkHash,
        bytes calldata _circuitOutput
    ) internal override {
        require(vkHash == _vkHash, "invalid vk");

        // FIXME:
        (uint64 _minBlockNum, uint248 _sum) = _decodeOutput(_circuitOutput);
        
        // other logic that uses the proven data...
        _minBlockNum;
        _sum;
    }

    // Suppose in the app circuit you have:
    // api.OutputUint(64, minBlockNum)
    // api.OutputUint(248, sum)
    // Then, we can decode the output the following way
    function _decodeOutput(
        bytes calldata output
    ) internal pure returns (uint64, uint248) {
        uint64 minBlockNum = uint64(bytes8(output[0:8])); 
        uint248 sum = uint248(bytes31(output[8:8+31])); 
        return (minBlockNum, sum);
    }

    function setVkHash(bytes32 _vkHash) external onlyOwner {
        vkHash = _vkHash;
    }

    function getHookPermissions()
        public
        pure
        override
        returns (Hooks.Permissions memory)
    {
        return
            Hooks.Permissions({
                beforeInitialize: false,
                afterInitialize: false,
                beforeAddLiquidity: false,
                beforeRemoveLiquidity: false,
                afterAddLiquidity: false,
                afterRemoveLiquidity: false,
                beforeSwap: true,
                afterSwap: false,
                beforeDonate: false,
                afterDonate: false,
                beforeSwapReturnDelta: false,
                afterSwapReturnDelta: false,
                afterAddLiquidityReturnDelta: false,
                afterRemoveLiquidityReturnDelta: false
            });
    }

    function beforeSwap(
        address,
        PoolKey calldata key,
        IPoolManager.SwapParams calldata,
        bytes calldata
    )
        external
        override
        onlyPoolManager
        returns (bytes4, BeforeSwapDelta, uint24)
    {
        // FIXME: Dynamic fee

        // uint24 fee = getFee();
        // poolManager.updateDynamicLPFee(key, fee);
        return (this.beforeSwap.selector, BeforeSwapDeltaLibrary.ZERO_DELTA, 0);
    }

}
