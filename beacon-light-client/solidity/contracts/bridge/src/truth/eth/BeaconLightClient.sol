// SPDX-License-Identifier: MIT
pragma solidity 0.8.9;

import '../../utils/LightClientUpdateVerifier.sol';
import '../../interfaces/ILightClient.sol';

uint256 constant BUFER_SIZE = 32;

contract BeaconLightClient is LightClientUpdateVerifier, ILightClient {
  struct LightClientUpdate {
    bytes32 attestedHeaderRoot;
    uint256 attestedHeaderSlot;
    bytes32 finalizedHeaderRoot;
    bytes32 finalizedExecutionStateRoot;
    uint256[2] a;
    uint256[2][2] b;
    uint256[2] c;
  }

  bytes32[BUFER_SIZE] public optimisticHeaders;

  uint256[BUFER_SIZE] public optimisticSlots;

  bytes32[BUFER_SIZE] public finalizedHeaders;

  bytes32[BUFER_SIZE] public executionStateRoots;

  uint256 public currentIndex;

  constructor(
    bytes32 _optimisticHeaderRoot,
    uint256 _optimisticHeaderSlot,
    bytes32 _finalizedHeaderRoot,
    bytes32 _executionStateRoot
  ) {
    currentIndex = 0;

    optimisticHeaders[currentIndex] = _optimisticHeaderRoot;
    optimisticSlots[currentIndex] = _optimisticHeaderSlot;
    finalizedHeaders[currentIndex] = _finalizedHeaderRoot;
    executionStateRoots[currentIndex] = _executionStateRoot;
  }

  function optimisticHeaderRoot() public view returns (bytes32) {
    return optimisticHeaders[currentIndex];
  }

  function optimisticHeaderSlot() public view returns (uint256) {
    return optimisticSlots[currentIndex];
  }

  function finalizedHeaderRoot() public view returns (bytes32) {
    return finalizedHeaders[currentIndex];
  }

  function executionStateRoot() public view returns (bytes32) {
    return executionStateRoots[currentIndex];
  }

  // TODO: fix
  function light_client_update(LightClientUpdate calldata update)
    external
    payable
  {
    require(
      verifyUpdate(
        update.a,
        update.b,
        update.c,
        optimisticHeaderRoot(),
        update.attestedHeaderRoot,
        update.attestedHeaderSlot,
        update.finalizedHeaderRoot,
        update.finalizedExecutionStateRoot
      ),
      '!proof'
    );

    currentIndex = (currentIndex + 1) % BUFER_SIZE;

    optimisticHeaders[currentIndex] = update.attestedHeaderRoot;
    optimisticSlots[currentIndex] = update.attestedHeaderSlot;
    finalizedHeaders[currentIndex] = update.finalizedHeaderRoot;
    executionStateRoots[currentIndex] = update.finalizedExecutionStateRoot;
  }
}
