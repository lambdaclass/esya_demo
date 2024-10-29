// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "forge-std/Script.sol";
import "../src/MerkleRootStorage.sol";

/// @title MerkleRootStorageScript
/// @dev Script to deploy the MerkleRootStorage contract and set a sample Merkle root
contract MerkleRootStorageScript is Script {

    function run() external {
        // Start broadcasting transactions to Anvil or a local network
        vm.startBroadcast();

        // Deploy the MerkleRootStorage contract
        MerkleRootStorage merkleRootStorage = new MerkleRootStorage();
        console.log("MerkleRootStorage deployed at:", address(merkleRootStorage));

        // Stop broadcasting
        vm.stopBroadcast();
    }
}