// SPDX-License-Identifier: MIT
pragma solidity 0.8.21;

import {Script} from "forge-std/Script.sol";
import {LiquidityManager} from "../src/LiqManager.sol";

contract LiqManagerScript is Script {
    function run() public {
        address token0 = vm.envAddress("TOKEN_A");
        address token1 = vm.envAddress("TOKEN_B");
        address pool = vm.envAddress("POOL_ADDRESS");

        vm.startBroadcast();

        LiquidityManager liqManager = new LiquidityManager(token0, token1, pool);

        vm.stopBroadcast();
    }
}
