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
interface IUniswapCurve {
    function forcastVolatility(
        uint256 new_volatility
    ) external returns (uint256);

    error CurveCustomError();
}

contract AutoPilotHook is BaseHook, BrevisApp, Ownable {
    using LPFeeLibrary for uint24;

    uint256 public constant NEXT_BLOCK_THRESHOLD = 900;

    uint256 public constant BASE_FEE = 3000; // 0.3%
    uint256 public constant HIGH_VOLATILITY_FEE = 6000; // 0.6%

    bytes32 public vkHash;

    IUniswapCurve _dynamicFee;

    uint256 lastVolatility;
    uint256 lastBlock;
    uint256 forcastVolatility;

    error MustUseDynamicFee();

    constructor(
        IPoolManager _poolManager,
        address brevisRequest,
        address stylusMLContract
    ) BaseHook(_poolManager) BrevisApp(brevisRequest) Ownable(msg.sender) {
        _dynamicFee = IUniswapCurve(stylusMLContract);
    }

    // BrevisQuery contract will call our callback once Brevis backend submits the proof.
    function handleProofResult(
        // solhint-disable-line private-vars-leading-underscore
        bytes32 _vkHash,
        bytes calldata _circuitOutput
    ) internal override {
        require(vkHash == _vkHash, "invalid vk");
        require(
            block.timestamp > lastBlock + NEXT_BLOCK_THRESHOLD,
            "invalid threshold time"
        );

        // Extract the volatility
        lastVolatility = _decodeOutput(_circuitOutput);
        lastBlock = block.timestamp;

        // Compute the forcast volatility
        forcastVolatility = _dynamicFee.forcastVolatility(lastVolatility);
    }

    // Decode the following input
    // api.OutputUint(248, std)
    function _decodeOutput(
        bytes calldata output
    ) internal pure returns (uint256) {
        uint256 volatility = uint256(bytes31(output[0:31]));
        return volatility;
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
                beforeInitialize: true,
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

    function beforeInitialize(
        address,
        PoolKey calldata key,
        uint160
    ) external pure override returns (bytes4) {
        // Check that the attached pool has dynamic fee
        if (!key.fee.isDynamicFee()) revert MustUseDynamicFee();
        return this.beforeInitialize.selector;
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
        uint24 fee = BASE_FEE;

        // Depending of the volatility, apply different fees
        if (forcastVolatility > 5000) {
            // For improvement, we can use the magnitude of the forcast to determine the price of the fee
            fee = HIGH_VOLATILITY_FEE;
        }

        poolManager.updateDynamicLPFee(key, fee);
        return (this.beforeSwap.selector, BeforeSwapDeltaLibrary.ZERO_DELTA, 0);
    }
}
