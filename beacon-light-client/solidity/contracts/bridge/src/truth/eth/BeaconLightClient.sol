// SPDX-License-Identifier: MIT
pragma solidity 0.8.20;

import '../../utils/LightClientUpdateVerifier.sol';
import '../../interfaces/ILightClient.sol';
import '@openzeppelin/contracts/access/Ownable.sol';

uint256 constant BUFER_SIZE = 32;

contract BeaconLightClient is Ownable, LightClientUpdateVerifier, ILightClient {
  error ProofVerificationFailed();
  error InvalidAccessControll();

  bytes32[BUFER_SIZE] public optimisticHeaders;

  uint256[BUFER_SIZE] public optimisticSlots;

  bytes32[BUFER_SIZE] public finalizedHeaders;

  bytes32[BUFER_SIZE] public executionStateRoots;

  uint256 public currentIndex;

  bytes32 domain;

  address adapterAddress;

  constructor(
    bytes32 _optimisticHeaderRoot,
    uint256 _optimisticHeaderSlot,
    bytes32 _finalizedHeaderRoot,
    bytes32 _executionStateRoot,
    bytes32 _domain
  ) Ownable(msg.sender) {
    currentIndex = 0;

    optimisticHeaders[currentIndex] = _optimisticHeaderRoot;
    optimisticSlots[currentIndex] = _optimisticHeaderSlot;
    finalizedHeaders[currentIndex] = _finalizedHeaderRoot;
    executionStateRoots[currentIndex] = _executionStateRoot;
    domain = _domain;
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

  function changeAdapterAddress(address _adapterAddress) external onlyOwner {
    adapterAddress = _adapterAddress;
  }

  function lightClientUpdate(LightClientUpdate calldata update) external {
    if (msg.sender != adapterAddress && msg.sender != owner()) {
      revert InvalidAccessControll();
    }

    if (
      !verifyUpdate(
        update.a,
        update.b,
        update.c,
        optimisticHeaderRoot(),
        update.attestedHeaderRoot,
        update.attestedHeaderSlot,
        update.finalizedHeaderRoot,
        update.finalizedExecutionStateRoot,
        domain
      )
    ) {
      revert ProofVerificationFailed();
    }

    currentIndex = (currentIndex + 1) % BUFER_SIZE;

    optimisticHeaders[currentIndex] = update.attestedHeaderRoot;
    optimisticSlots[currentIndex] = update.attestedHeaderSlot;
    finalizedHeaders[currentIndex] = update.finalizedHeaderRoot;
    executionStateRoots[currentIndex] = update.finalizedExecutionStateRoot;
  }
}
