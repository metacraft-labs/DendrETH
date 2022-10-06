// SPDX-License-Identifier: MIT
pragma solidity ^0.8;

abstract contract GasReportable {
  event LogLine(uint256 x, uint256 y, uint256 z);
  event LogStart(uint256 y, uint256 z);
  event LogEnd(uint256 y, uint256 z);

  modifier gas_report(uint256 f) {
    emit LogStart(f, gasleft());
    _;
    emit LogEnd(f, gasleft());
  }
}
