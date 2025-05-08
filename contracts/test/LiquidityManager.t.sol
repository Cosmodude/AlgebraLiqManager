// SPDX-License-Identifier: GPL-2.0-or-later
pragma solidity ^0.8.21;

import "forge-std/Test.sol";
import "../src/LiqController.sol";
import "./mocks/MockToken.sol";
import "./mocks/MockAlgebraPool.sol";

contract LiquidityManagerTest is Test {
    LiquidityManager public liquidityManager;
    MockToken public token0;
    MockToken public token1;
    MockAlgebraPool public pool;
    address public owner;
    address public user;

    function setUp() public {
        owner = makeAddr("owner");
        user = makeAddr("user");
        
        // Deploy mock tokens
        token0 = new MockToken("Token0", "TK0");
        token1 = new MockToken("Token1", "TK1");
        
        // Deploy mock pool
        pool = new MockAlgebraPool(address(token0), address(token1));
        
        // Deploy liquidity manager
        vm.prank(owner);
        liquidityManager = new LiquidityManager(address(token0), address(token1), address(pool));
        
        // Transfer tokens to liquidity manager
        token0.transfer(address(liquidityManager), 10000 * 10**18);
        token1.transfer(address(liquidityManager), 10000 * 10**18);
    }

    function test_Constructor() public view {
        assertEq(liquidityManager.token0(), address(token0));
        assertEq(liquidityManager.token1(), address(token1));
        assertEq(liquidityManager.pool(), address(pool));
        assertEq(liquidityManager.owner(), owner);
    }

    function test_ConstructorTokenOrdering() public {
        // Create tokens with specific addresses to control ordering
        MockToken tokenA = new MockToken("TokenA", "TKA");
        MockToken tokenB = new MockToken("TokenB", "TKB");
        
        // Get the addresses
        address addrA = address(tokenA);
        address addrB = address(tokenB);
        
        // Deploy pool with tokens in specific order
        MockAlgebraPool newPool = new MockAlgebraPool(addrA, addrB);
        
        // Deploy liquidity manager
        vm.prank(owner);
        LiquidityManager newManager = new LiquidityManager(addrA, addrB, address(newPool));
        
        // Verify that token0 is the token with lower address value
        address expectedToken0 = addrA < addrB ? addrA : addrB;
        address expectedToken1 = addrA < addrB ? addrB : addrA;
        
        assertEq(newManager.token0(), expectedToken0, "token0 should be the token with lower address");
        assertEq(newManager.token1(), expectedToken1, "token1 should be the token with higher address");
        
        // Verify that pool and manager have the same token ordering
        assertEq(newManager.token0(), newPool.token0(), "Pool and manager should have same token0");
        assertEq(newManager.token1(), newPool.token1(), "Pool and manager should have same token1");
        
        // Log the addresses for clarity
        console.log("TokenA address:", addrA);
        console.log("TokenB address:", addrB);
        console.log("token0 address:", newManager.token0());
        console.log("token1 address:", newManager.token1());
    }

    function test_ProvideLiquidity() public {
        int24 bottomTick = -100;
        int24 topTick = 100;
        uint128 liquidityDesired = 1000;
        
        vm.startPrank(owner);
        liquidityManager.provideLiquidity(
            owner,
            bottomTick,
            topTick,
            liquidityDesired,
            ""
        );
        vm.stopPrank();
        
        assertEq(pool.positions(bottomTick, topTick), liquidityDesired);
    }

    function test_WithdrawLiquidity() public {
        int24 bottomTick = -100;
        int24 topTick = 100;
        uint128 liquidityDesired = 1000;
        uint256 amount0Min = 900 * 10**18;
        uint256 amount1Min = 900 * 10**18;
        
        // First provide liquidity
        vm.startPrank(owner);
        liquidityManager.provideLiquidity(
            owner,
            bottomTick,
            topTick,
            liquidityDesired,
            ""
        );
        
        // Then withdraw it
        liquidityManager.withdrawLiquidity(
            owner,
            bottomTick,
            topTick,
            liquidityDesired,
            amount0Min,
            amount1Min
        );
        vm.stopPrank();
        
        assertEq(pool.positions(bottomTick, topTick), 0);
    }

    function test_WithdrawToken() public {
        uint256 amount = 1000 * 10**18;
        
        vm.startPrank(owner);
        liquidityManager.withdrawToken(address(token0), amount);
        vm.stopPrank();
        
        assertEq(token0.balanceOf(owner), amount);
    }

    function test_WithdrawNative() public {
        // Send some ETH to the contract
        vm.deal(address(liquidityManager), 1 ether);
        
        uint256 initialBalance = owner.balance;
        
        vm.startPrank(owner);
        liquidityManager.withdrawNative();
        vm.stopPrank();
        
        assertEq(owner.balance, initialBalance + 1 ether);
        assertEq(address(liquidityManager).balance, 0);
    }

    function testFail_NonOwnerAccess() public {
        vm.prank(user);
        liquidityManager.withdrawToken(address(token0), 1000 * 10**18);
    }

    function testFail_WithdrawMoreThanAvailable() public {
        vm.startPrank(owner);
        liquidityManager.withdrawToken(address(token0), 20000 * 10**18);
        vm.stopPrank();
    }
} 