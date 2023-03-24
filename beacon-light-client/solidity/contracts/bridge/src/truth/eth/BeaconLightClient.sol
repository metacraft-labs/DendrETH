// SPDX-License-Identifier: MIT
pragma solidity 0.8.17;

import '../../utils/LightClientUpdateVerifier.sol';

uint256 constant BUFER_SIZE = 32;

contract BeaconLightClient is LightClientUpdateVerifier {
  struct LightClientUpdate {
    bytes32 attested_header_root;
    uint256 attested_header_slot;
    bytes32 finalized_header_root;
    bytes32 finalized_execution_state_root;
    uint256[2] a;
    uint256[2][2] b;
    uint256[2] c;
  }

  bytes32[BUFER_SIZE] public optimistic_headers;

  uint256[BUFER_SIZE] public optimistic_slots;

  bytes32[BUFER_SIZE] public finalized_headers;

  bytes32[BUFER_SIZE] public execution_state_roots;

  uint256 public currentIndex;

  constructor(
    bytes32 _optimistic_header_root,
    uint256 _optimistic_header_slot,
    bytes32 _finalized_header_root,
    bytes32 _execution_state_root
  ) {
    currentIndex = 0;

    optimistic_headers[currentIndex] = _optimistic_header_root;
    optimistic_slots[currentIndex] = _optimistic_header_slot;
    finalized_headers[currentIndex] = _finalized_header_root;
    execution_state_roots[currentIndex] = _execution_state_root;
  }

  function optimistic_header_root() public view returns (bytes32) {
    return optimistic_headers[currentIndex];
  }

  function optimistic_header_slot() public view returns (uint256) {
    return optimistic_slots[currentIndex];
  }

  function finalized_header_root() public view returns (bytes32) {
    return finalized_headers[currentIndex];
  }

  function execution_state_root() public view returns (bytes32) {
    return execution_state_roots[currentIndex];
  }

  function light_client_update(LightClientUpdate calldata update)
    external
    payable
  {
    require(
      verifyUpdate(
        update.a,
        update.b,
        update.c,
        optimistic_header_root(),
        update.attested_header_root,
        update.attested_header_slot,
        update.finalized_header_root,
        update.finalized_execution_state_root
      ),
      '!proof'
    );

    currentIndex = (currentIndex + 1) % BUFER_SIZE;

    optimistic_headers[currentIndex] = update.attested_header_root;
    optimistic_slots[currentIndex] = update.attested_header_slot;
    finalized_headers[currentIndex] = update.finalized_header_root;
    execution_state_roots[currentIndex] = update.finalized_execution_state_root;
  }
}
