// SPDX-License-Identifier: GPL-2.0-or-later
pragma solidity 0.8.21;

import "./interfaces/IAlgebraMintCallback.sol";
import "./interfaces/IAlgebraPool.sol";
import "./interfaces/IERC20.sol";
import "@uniswap/v3-periphery/libraries/TransferHelper.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

/// @title LiquidityManager
/// @notice Manages liquidity provision and allows the owner to withdraw tokens and native currency
contract LiquidityManager is IAlgebraMintCallback, Ownable {
    address public token0;
    address public token1;
    address public pool;

    constructor(address _token0, address _token1, address _pool) Ownable(msg.sender) {
        // Ensure token0 is the token with lower address
        (token0, token1) = _token0 < _token1 ? (_token0, _token1) : (_token1, _token0);
        pool = _pool;
    }

    /// @notice Adds liquidity to the Algebra pool
    /// @param recipient The address for which the liquidity will be created
    /// @param bottomTick The lower tick of the position
    /// @param topTick The upper tick of the position
    /// @param liquidityDesired The desired amount of liquidity to mint
    /// @param data Any data to be passed through to the callback
    function provideLiquidity(
        address recipient,
        int24 bottomTick,
        int24 topTick,
        uint128 liquidityDesired,
        bytes calldata data
    ) external onlyOwner {
        IAlgebraPool(pool).mint(address(this), recipient, bottomTick, topTick, liquidityDesired, data);
    }

    /// @notice Withdraws liquidity and fees from a position
    /// @param recipient The address that will receive the tokens
    /// @param bottomTick The lower tick of the position
    /// @param topTick The upper tick of the position
    /// @param liquidity The amount of liquidity to remove
    /// @param amount0Min The minimum amount of token0 that should be received
    /// @param amount1Min The minimum amount of token1 that should be received
    function withdrawLiquidity(
        address recipient,
        int24 bottomTick,
        int24 topTick,
        uint128 liquidity,
        uint256 amount0Min,
        uint256 amount1Min
    ) external onlyOwner {
        (uint256 amount0, uint256 amount1) = IAlgebraPool(pool).burn(
            bottomTick,
            topTick,
            liquidity,
            "" // empty bytes for data parameter
        );

        (uint128 collected0, uint128 collected1) =
            IAlgebraPool(pool).collect(recipient, bottomTick, topTick, type(uint128).max, type(uint128).max);

        require(amount0 + collected0 >= amount0Min, "Insufficient token0 amount");
        require(amount1 + collected1 >= amount1Min, "Insufficient token1 amount");
    }

    /// @notice Callback function called by the Algebra pool after minting liquidity
    /// @param amount0Owed The amount of token0 owed to the pool
    /// @param amount1Owed The amount of token1 owed to the pool
    function algebraMintCallback(uint256 amount0Owed, uint256 amount1Owed, bytes calldata) external override {
        require(msg.sender == pool, "Unauthorized callback");

        if (amount0Owed > 0) {
            TransferHelper.safeTransfer(token0, msg.sender, amount0Owed);
        }

        if (amount1Owed > 0) {
            TransferHelper.safeTransfer(token1, msg.sender, amount1Owed);
        }
    }

    /// @notice Allows the owner to withdraw specified ERC20 tokens from the contract
    /// @param token The address of the ERC20 token to withdraw
    /// @param amount The amount of tokens to withdraw
    function withdrawToken(address token, uint256 amount) external onlyOwner {
        TransferHelper.safeTransfer(token, owner(), amount);
    }

    /// @notice Allows the owner to withdraw all native currency from the contract
    function withdrawNative() external onlyOwner {
        uint256 balance = address(this).balance;
        require(balance > 0, "No native currency to withdraw");
        (bool success,) = owner().call{value: balance}("");
        require(success, "Native currency withdrawal failed");
    }

    // Fallback function to accept native currency
    receive() external payable {}

    /// @notice Updates the token and pool addresses
    /// @param newToken0 The new token0 address
    /// @param newToken1 The new token1 address
    /// @param newPool The new pool address
    function changeTokensAndPool(
        address newToken0,
        address newToken1,
        address newPool
    ) external onlyOwner {
        (token0, token1) = newToken0 < newToken1 ? (newToken0, newToken1) : (newToken1, newToken0);
        pool = newPool;
    }
}
