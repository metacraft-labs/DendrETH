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

  /// @notice Verifies the proof and writes the data for given slot if valid
  /// @param proof the zk proof for total value locked
  /// @param slot the slot for which the proof is ran
  /// @param balanceSum the sum of the balances of all validators with withdrawal credentials equal to WITHDRAWAL_CREDENTIALS
  /// @param _numberOfNonActivatedValidators number of validators yet to be activated
  /// @param _numberOfActiveValidators number of active validators
  /// @param _numberOfExitedValidators number of exited validators
  function verify(
    bytes calldata proof,
    uint256 slot,
    uint64 balanceSum,
    uint64 _numberOfNonActivatedValidators,
    uint64 _numberOfActiveValidators,
    uint64 _numberOfExitedValidators,
    uint64 _numberOfSlashedValidators
  ) external;
}
