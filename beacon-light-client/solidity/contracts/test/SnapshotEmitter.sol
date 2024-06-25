// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

contract SnapshotEmitter {
  event SnapshotTaken(uint256 era, uint256 slot);

  function emitSnapshot(uint256 era, uint256 slot) external {
    emit SnapshotTaken(era, slot);
  }
}
