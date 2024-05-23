// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

contract VerifierMock {
  bool public success = true;

  function setSuccess(bool _success) public {
    success = _success;
  }

  function Verify(
    bytes calldata,
    uint256[] calldata
  ) public view returns (bool) {
    return success;
  }
}
