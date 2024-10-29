// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "forge-std/Test.sol";
import "../src/MerkleRootStorage.sol";

/// @title MerkleRootStorageTest
/// @dev Test suite for the MerkleRootStorage contract
contract MerkleRootStorageTest is Test {
    MerkleRootStorage public merkleRootStorage;

    address owner = address(this);    // Test contract address is the owner
    address nonOwner = address(0x1234); // Some arbitrary non-owner address

    function setUp() public {
        merkleRootStorage = new MerkleRootStorage();
    }

    /// @dev Test setting and getting a Merkle root as the owner
    function testSetAndGetMerkleRoot() public {
        string memory key = "key1";
        bytes32 merkleRoot = keccak256(abi.encodePacked("test merkle root"));

        // Set Merkle root as the owner
        merkleRootStorage.setMerkleRoot(key, merkleRoot);

        // Assert the Merkle root is correctly stored and retrievable
        bytes32 retrievedMerkleRoot = merkleRootStorage.getMerkleRoot(key);
        assertEq(retrievedMerkleRoot, merkleRoot, "Merkle root should match the stored value");
    }

    /// @dev Test that a non-owner cannot set a Merkle root
    function testNonOwnerCannotSetMerkleRoot() public {
        vm.startPrank(nonOwner);

        string memory key = "key2";
        bytes32 merkleRoot = keccak256(abi.encodePacked("test merkle root"));

        // Expect custom error `OwnableUnauthorizedAccount` when a non-owner tries to set a Merkle root
        vm.expectRevert(abi.encodeWithSignature("OwnableUnauthorizedAccount(address)", nonOwner));
        merkleRootStorage.setMerkleRoot(key, merkleRoot);

        vm.stopPrank();
    }

    /// @dev Test that anyone can read a Merkle root
    function testAnyoneCanReadMerkleRoot() public {
        string memory key = "key3";
        bytes32 merkleRoot = keccak256(abi.encodePacked("public merkle root"));

        // Set Merkle root as the owner
        merkleRootStorage.setMerkleRoot(key, merkleRoot);

        // Read as non-owner
        vm.prank(nonOwner);
        bytes32 retrievedMerkleRoot = merkleRootStorage.getMerkleRoot(key);

        // Assert the Merkle root is readable and correct
        assertEq(retrievedMerkleRoot, merkleRoot, "Non-owner should be able to read the Merkle root");
    }
}