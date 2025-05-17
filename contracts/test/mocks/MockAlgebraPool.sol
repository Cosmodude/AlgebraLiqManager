// SPDX-License-Identifier: GPL-2.0-or-later
pragma solidity ^0.8.21;

import "../../src/interfaces/IAlgebraPool.sol";
import "../../src/interfaces/IAlgebraMintCallback.sol";

// Mock Algebra Pool for testing
contract MockAlgebraPool is IAlgebraPool {
    address public immutable override token0;
    address public immutable override token1;
    mapping(int24 => mapping(int24 => uint128)) public positions;
    mapping(int24 => mapping(int24 => uint256)) public fees0;
    mapping(int24 => mapping(int24 => uint256)) public fees1;

    constructor(address _token0, address _token1) {
        // Ensure token0 is the token with lower address
        (token0, token1) = _token0 < _token1 ? (_token0, _token1) : (_token1, _token0);
    }

    function mint(address, address, int24 bottomTick, int24 topTick, uint128 liquidityDesired, bytes calldata data)
        external
        override
        returns (uint256 amount0, uint256 amount1, uint128 liquidityActual)
    {
        positions[bottomTick][topTick] += liquidityDesired;
        liquidityActual = liquidityDesired;
        amount0 = 1000 * 10 ** 18; // Mock amounts
        amount1 = 1000 * 10 ** 18;

        IAlgebraMintCallback(msg.sender).algebraMintCallback(amount0, amount1, data);
    }

    function burn(int24 bottomTick, int24 topTick, uint128 amount, bytes calldata)
        external
        override
        returns (uint256 amount0, uint256 amount1)
    {
        require(positions[bottomTick][topTick] >= amount, "Insufficient liquidity");
        positions[bottomTick][topTick] -= amount;
        amount0 = 1000 * 10 ** 18; // Mock amounts
        amount1 = 1000 * 10 ** 18;
    }

    function collect(address, int24 bottomTick, int24 topTick, uint128, uint128)
        external
        override
        returns (uint128 amount0, uint128 amount1)
    {
        amount0 = uint128(fees0[bottomTick][topTick]);
        amount1 = uint128(fees1[bottomTick][topTick]);
        fees0[bottomTick][topTick] = 0;
        fees1[bottomTick][topTick] = 0;
    }

    // Mock functions to satisfy interface
    function safelyGetStateOfAMM() external pure returns (uint160, int24, uint16, uint8, uint128, int24, int24) {
        return (0, 0, 0, 0, 0, 0, 0);
    }

    function isUnlocked() external pure returns (bool) {
        return true;
    }

    function globalState() external pure returns (uint160, int24, uint16, uint8, uint16, bool) {
        return (0, 0, 0, 0, 0, true);
    }

    function tickSpacing() external pure returns (int24) {
        return 10;
    }
}
