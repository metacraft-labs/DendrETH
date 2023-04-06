// SPDX-License-Identifier: MIT
pragma solidity 0.8.9;

struct LightClientUpdate {
  bytes32 attested_header_root;
  bytes32 finalized_header_root;
  bytes32 finalized_execution_state_root;
  uint256[2] a;
  uint256[2][2] b;
  uint256[2] c;
}
