// SPDX-License-Identifier: MIT
pragma solidity 0.8.9;

import './MerkleProof.sol';

contract BeaconChain is MerkleProof {
  struct BeaconBlockHeader {
    uint64 slot;
    uint64 proposer_index;
    bytes32 parent_root;
    bytes32 state_root;
    bytes32 body_root;
  }

  function hash_tree_root(BeaconBlockHeader memory beacon_header)
    internal
    pure
    returns (bytes32)
  {
    bytes32[] memory leaves = new bytes32[](5);
    leaves[0] = bytes32(to_little_endian_64(beacon_header.slot));
    leaves[1] = bytes32(to_little_endian_64(beacon_header.proposer_index));
    leaves[2] = beacon_header.parent_root;
    leaves[3] = beacon_header.state_root;
    leaves[4] = beacon_header.body_root;
    return merkle_root(leaves);
  }

  function merkle_root(bytes32[] memory leaves)
    internal
    pure
    returns (bytes32)
  {
    uint256 len = leaves.length;
    if (len == 0) return bytes32(0);
    else if (len == 1) return hash(abi.encodePacked(leaves[0]));
    else if (len == 2) return hash_node(leaves[0], leaves[1]);
    uint256 bottom_length = get_power_of_two_ceil(len);
    bytes32[] memory o = new bytes32[](bottom_length * 2);
    for (uint256 i = 0; i < len; ++i) {
      o[bottom_length + i] = leaves[i];
    }
    for (uint256 i = bottom_length - 1; i > 0; --i) {
      o[i] = hash_node(o[i * 2], o[i * 2 + 1]);
    }
    return o[1];
  }

  //  Get the power of 2 for given input, or the closest higher power of 2 if the input is not a power of 2.
  function get_power_of_two_ceil(uint256 x) internal pure returns (uint256) {
    if (x <= 1) return 1;
    else if (x == 2) return 2;
    else return 2 * get_power_of_two_ceil((x + 1) >> 1);
  }

  function to_little_endian_64(uint64 value) internal pure returns (bytes8 r) {
    return bytes8(reverse64(value));
  }

  function reverse64(uint64 input) internal pure returns (uint64 v) {
    v = input;

    // swap bytes
    v = ((v & 0xFF00FF00FF00FF00) >> 8) | ((v & 0x00FF00FF00FF00FF) << 8);

    // swap 2-byte long pairs
    v = ((v & 0xFFFF0000FFFF0000) >> 16) | ((v & 0x0000FFFF0000FFFF) << 16);

    // swap 4-byte long pairs
    v = (v >> 32) | (v << 32);
  }
}
