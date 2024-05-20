// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import {BalanceVerifier} from './BalanceVerifier.sol';
import {IBalanceVerifierLido} from './interfaces/IBalanceVerifierLido.sol';

contract BalanceVerifierLido is BalanceVerifier, IBalanceVerifierLido {
  mapping(uint256 => Report) reports;

  constructor(
    uint256 verifierDigest,
    bytes32 withdrawalCredentials,
    uint256 genesisBlockTimestamp
  )
    BalanceVerifier(
      verifierDigest,
      withdrawalCredentials,
      genesisBlockTimestamp
    )
  {}

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
  ) external override {
    _verify(
      proof,
      slot,
      balanceSum,
      _numberOfNonActivatedValidators,
      _numberOfActiveValidators,
      _numberOfExitedValidators,
      _numberOfSlashedValidators
    );

    uint64 numValidators = _numberOfNonActivatedValidators +
      _numberOfActiveValidators +
      _numberOfExitedValidators;

    reports[slot] = Report({
      present: true,
      cBalanceGwei: balanceSum,
      numValidators: numValidators,
      exitedValidators: _numberOfExitedValidators
    });

    emit ReportAdded(
      slot,
      balanceSum,
      numValidators,
      _numberOfExitedValidators
    );
  }

  function getReport(
    uint256 slot
  )
    external
    view
    override
    returns (
      bool /* success */,
      uint256 /* clBalanceGwei */,
      uint256 /* numValidators */,
      uint256 /* exitedValidators */
    )
  {
    Report memory report = reports[slot];

    return (
      report.present,
      report.cBalanceGwei,
      report.numValidators,
      report.exitedValidators
    );
  }
}
