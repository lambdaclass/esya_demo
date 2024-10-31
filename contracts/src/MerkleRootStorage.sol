// SPDX-License-Identifier: MIT
pragma solidity ^0.8.0;

import "openzeppelin-contracts/contracts/access/Ownable.sol";

/// @title MerkleRootStorage
/// @dev A contract for storing Merkle root hashes in a mapping, where only the owner can set them, and anyone can read them.
contract MerkleRootStorage is Ownable {

    // Mapping to store Merkle roots with string keys
    mapping(string => bytes32) private merkleRoots;

    constructor() Ownable(msg.sender) {}

    /// @notice Sets a Merkle root associated with a given key
    /// @dev Only the owner can set a Merkle root
    /// @param key The key to associate with the Merkle root
    /// @param merkleRoot The Merkle root to store
    function setMerkleRoot(string memory key, bytes32 merkleRoot) external onlyOwner {
        merkleRoots[key] = merkleRoot;
    }

    /// @notice Reads the Merkle root associated with a given key
    /// @param key The key to retrieve the Merkle root for
    /// @return The Merkle root associated with the provided key
    function getMerkleRoot(string memory key) external view returns (bytes32) {
        return merkleRoots[key];
    }
}
