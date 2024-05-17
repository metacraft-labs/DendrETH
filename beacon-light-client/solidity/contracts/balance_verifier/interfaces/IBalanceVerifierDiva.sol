// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import {IBalanceVerifier} from './IBalanceVerifier.sol';

interface IBalanceVerifierDiva is IBalanceVerifier {
  struct Report {
    bool present;
    uint64 cBalanceGwei;
    uint64 numValidators;
    uint64 exitedValidators;
    uint64 slashedValidators;
  }

  event ReportAdded(
    uint256 indexed slot,
    uint64 clBalanceGwei,
    uint64 numValidators,
    uint64 exitedValidators,
    uint64 slashedValidators
  );

  function getReport(
    uint256 slot
  )
    external
    view
    returns (
      bool success,
      uint64 clBalanceGwei,
      uint64 numValidators,
      uint64 exitedValidators,
      uint64 slashedValidators
    );
}
