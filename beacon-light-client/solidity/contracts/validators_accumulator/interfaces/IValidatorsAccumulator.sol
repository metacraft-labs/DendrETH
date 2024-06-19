// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

interface IValidatorsAccumulator {
  event Deposited(bytes pubkey);

  error InvalidCaller();

  function getValidatorsAccumulator() external view returns (bytes32 node);

  function deposit(
    bytes calldata pubkey,
    bytes calldata withdrawalCredentials,
    bytes calldata signature,
    bytes32 depositDataRoot
  ) external payable;

  function findAccumulatorByBlock(
    uint64 blockNumber
  ) external view returns (bytes32 accumulator);
}
