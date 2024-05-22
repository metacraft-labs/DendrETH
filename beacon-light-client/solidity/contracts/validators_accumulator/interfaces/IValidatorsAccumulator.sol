// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

interface IValidatorsAccumulator {
  event Deposited(
    bytes pubkey,
    bytes signature,
    bytes32 depositMessageRoot,
    bytes depositIndex
  );

  function get_validators_accumulator() external view returns (bytes32 node);

  function deposit(
    bytes calldata pubkey,
    bytes calldata withdrawalCredentials,
    bytes calldata signature,
    bytes32 depositDataRoot
  ) external payable;
}
