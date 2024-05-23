// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

interface IBalanceVerifier {
  /// @dev Beacon root out of range
  error BeaconRootOutOfRange();

  /// @dev No block root is found using the beacon roots precompile.
  error NoBlockRootFound();

  /// @dev Verification call failed
  error VerificationCallFailed();

  /// @dev Verification failed
  error VerificationFailed();

  function setVerifier(address newVerifier) external;
}
