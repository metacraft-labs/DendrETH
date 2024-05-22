// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import {BalanceVerifier} from './BalanceVerifier.sol';
import {IBalanceVerifierDiva} from './interfaces/IBalanceVerifierDiva.sol';

contract BalanceVerifierDiva is BalanceVerifier, IBalanceVerifierDiva {
  mapping(uint256 => Report) reports;

  constructor(
    uint256 verifierDigest,
    bytes32 withdrawalcredentials,
    uint256 genesisBlockTimestamp,
    address _verifier,
    address _owner
  )
    BalanceVerifier(
      verifierDigest,
      withdrawalcredentials,
      genesisBlockTimestamp,
      _verifier,
      _owner
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

    uint64 numValidators = _numberOfActiveValidators +
      _numberOfExitedValidators;

    reports[slot] = Report({
      present: true,
      cBalanceGwei: balanceSum,
      numValidators: numValidators,
      exitedValidators: _numberOfExitedValidators,
      slashedValidators: _numberOfSlashedValidators
    });

    emit ReportAdded(
      slot,
      balanceSum,
      numValidators,
      _numberOfExitedValidators,
      _numberOfSlashedValidators
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
      uint64 /* clBalanceGwei */,
      uint64 /* numValidators */,
      uint64 /* exitedValidators */,
      uint64 /* slashedValidators */
    )
  {
    Report memory report = reports[slot];

    return (
      report.present,
      report.cBalanceGwei,
      report.numValidators,
      report.exitedValidators,
      report.slashedValidators
    );
  }
}
