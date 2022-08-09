// SPDX-License-Identifier: MIT

pragma solidity 0.8.9;

contract MerkleProof {
    // Check if ``leaf`` at ``index`` verifies against the Merkle ``root`` and ``branch``.
    function is_valid_merkle_branch(
        bytes32 leaf,
        bytes32[] memory branch,
        uint64 depth,
        uint64 index,
        bytes32 root
    ) internal pure returns (bool) {
        bytes32 value = leaf;
        for (uint i = 0; i < depth; ++i) {
            if ((index / (2**i)) % 2 == 1) {
                value = hash_node(branch[i], value);
            } else {
                value = hash_node(value, branch[i]);
            }
        }
        return value == root;
    }

    function hash_node(bytes32 left, bytes32 right)
        internal
        pure
        returns (bytes32)
    {
        return hash(abi.encodePacked(left, right));
    }

    function hash(bytes memory value) internal pure returns (bytes32) {
        return sha256(value);
    }
}
