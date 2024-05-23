// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import {BalanceVerifier} from './BalanceVerifier.sol';
import {IBalanceVerifierLido} from './interfaces/IBalanceVerifierLido.sol';

contract BalanceVerifierLido is BalanceVerifier, IBalanceVerifierLido {
  /// @notice lido validators withdrawal credentials
  bytes32 public immutable WITHDRAWAL_CREDENTIALS;

  mapping(uint256 => Report) internal reports;

  constructor(
    uint256 verifierDigest,
    bytes32 withdrawalCredentials,
    uint256 genesisBlockTimestamp,
    address _verifier,
    address _owner
  ) BalanceVerifier(verifierDigest, genesisBlockTimestamp, _verifier, _owner) {
    WITHDRAWAL_CREDENTIALS = withdrawalCredentials;
  }

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
    uint256[] memory publicInputs = new uint256[](2);
    publicInputs[0] = VERIFIER_DIGEST;
    publicInputs[1] = (uint256(
      sha256(
        abi.encodePacked(
          _findBlockRoot(slot),
          WITHDRAWAL_CREDENTIALS,
          balanceSum,
          _numberOfNonActivatedValidators,
          _numberOfActiveValidators,
          _numberOfExitedValidators,
          _numberOfSlashedValidators
        )
      )
    ) & ((1 << 253) - 1));

    _verify(proof, publicInputs);

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
