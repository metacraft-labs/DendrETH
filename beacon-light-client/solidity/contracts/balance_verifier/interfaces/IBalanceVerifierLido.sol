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
}
