// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import {IBalanceVerifier} from './IBalanceVerifier.sol';

interface IBalanceVerifierLido is IBalanceVerifier {
  struct Report {
    bool present;
    uint64 cBalanceGwei;
    uint64 numValidators;
    uint64 exitedValidators;
  }

  event ReportAdded(
    uint256 slot,
    uint64 balanceSum,
    uint64 numValidators,
    uint64 exitedValidators
  );

  function getReport(
    uint256 slot
  )
    external
    view
    returns (
      bool success,
      uint256 clBalanceGwei,
      uint256 numValidators,
      uint256 exitedValidators
    );

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
