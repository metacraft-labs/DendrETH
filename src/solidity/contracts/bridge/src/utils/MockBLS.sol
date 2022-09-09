// SPDX-License-Identifier: MIT
pragma solidity 0.8.9;

import './BLSVerify.sol';

contract MockBLS is BLSVerify {
  function fast_aggregate_verify(
    bytes[] calldata,
    bytes calldata,
    bytes calldata
  ) external pure returns (bool) {
    return true;
  }

  function verifyProof(
    uint256[2] calldata,
    uint256[2][2] calldata,
    uint256[2] calldata,
    uint256[70] calldata
  ) public pure override returns (bool r) {
    return true;
  }
}
