// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

interface IValidatorsAccumulator {
  struct DepositData {
    uint256 blockNumber;
    bytes32 accumulator;
  }

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
    uint256 blockNumber
  ) external view returns (uint256 validatorsCount, bytes32 accumulator);
}
