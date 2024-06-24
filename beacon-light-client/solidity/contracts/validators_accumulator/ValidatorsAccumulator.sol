// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import {IDeposit} from './interfaces/IDeposit.sol';
import {IValidatorsAccumulator} from './interfaces/IValidatorsAccumulator.sol';

contract ValidatorsAccumulator is IValidatorsAccumulator {
  // The depth of the validator accumulator tree
  uint8 internal constant VALIDATOR_ACCUMULATOR_TREE_DEPTH = 32;
  address internal immutable depositAddress;
  uint256 internal constant MAX_VALUE = type(uint256).max;

  // An array to hold the branch hashes for the Merkle tree
  bytes32[VALIDATOR_ACCUMULATOR_TREE_DEPTH] internal branch;
  bytes32[VALIDATOR_ACCUMULATOR_TREE_DEPTH] internal zeroHashes;

  // A counter for the total number of validators
  uint256 internal validatorsCount;

  mapping(uint256 => bytes32) internal snapshots;
  uint256[] internal blockNumbers;

  constructor(address _depositAddress) {
    depositAddress = _depositAddress;

    // Compute hashes in empty Merkle tree
    for (
      uint256 height = 0;
      height < VALIDATOR_ACCUMULATOR_TREE_DEPTH - 1;
      height++
    )
      zeroHashes[height + 1] = sha256(
        abi.encodePacked(zeroHashes[height], zeroHashes[height])
      );
  }

  // Function to calculate and return the Merkle accumulator root of the validators
  function getValidatorsAccumulator() external view override returns (bytes32) {
    return _getRoot(validatorsCount);
  }

  // Function to handle deposits from validators
  function deposit(
    bytes calldata pubkey,
    bytes calldata withdrawalCredentials,
    bytes calldata signature,
    bytes32 depositDataRoot
  ) external payable override {
    // Perform the deposit using the DepositContract
    IDeposit(depositAddress).deposit{value: msg.value}(
      pubkey,
      withdrawalCredentials,
      signature,
      depositDataRoot
    );

    emit Deposited(pubkey);

    // Create a node for the validator
    bytes32 node = sha256(pubkey);

    validatorsCount += 1;

    // Insert the node into the Merkle accumulator tree
    uint256 _validatorsCount = validatorsCount;
    uint256 size = _validatorsCount;
    for (
      uint256 height = 0;
      height < VALIDATOR_ACCUMULATOR_TREE_DEPTH;
      height++
    ) {
      if ((size & 1) == 1) {
        branch[height] = node;
        break;
      }
      node = sha256(abi.encodePacked(branch[height], node));
      size /= 2;
    }

    uint256 blockNumber = block.number;
    snapshots[blockNumber] = _getRoot(_validatorsCount);
    uint256 blockNumbersLength = blockNumbers.length;
    if (
      blockNumbersLength == 0 ||
      blockNumbers[blockNumbersLength - 1] != blockNumber
    ) {
      blockNumbers.push(blockNumber);
    }
  }

  function findAccumulatorByBlock(
    uint256 blockNumber
  ) external view override returns (bytes32) {
    if (blockNumbers.length == 0) {
      return (zeroHashes[VALIDATOR_ACCUMULATOR_TREE_DEPTH - 1]);
    }

    uint256 foundBlockNumber = _binarySearchBlock(blockNumber);

    if (foundBlockNumber == MAX_VALUE) {
      return (zeroHashes[VALIDATOR_ACCUMULATOR_TREE_DEPTH - 1]);
    }

    return snapshots[foundBlockNumber];
  }

  function _getRoot(uint256 size) internal view returns (bytes32 node) {
    for (
      uint256 height = 0;
      height < VALIDATOR_ACCUMULATOR_TREE_DEPTH;
      height++
    ) {
      // This if-else structure supports tree balancing
      // If size is odd, the new node will be hashed with the previous node on this level
      // If size is even, the new node will be hashed with a predefined zero hash
      if ((size & 1) == 1) {
        node = sha256(abi.encodePacked(branch[height], node));
      } else {
        node = sha256(abi.encodePacked(node, zeroHashes[height]));
      }

      size /= 2;
    }
  }

  function _binarySearchBlock(
    uint256 blockNumber
  ) internal view returns (uint256) {
    uint256 lower;
    uint256 upper = blockNumbers.length - 1;

    uint256 upperBlockNumber = blockNumbers[upper];
    if (upperBlockNumber <= blockNumber) {
      return upperBlockNumber;
    }

    if (blockNumbers[lower] > blockNumber) {
      return MAX_VALUE;
    }

    while (upper > lower) {
      uint256 index = upper - (upper - lower) / 2; // ceil, avoiding overflow
      uint256 indexBlockNumber = blockNumbers[index];
      if (indexBlockNumber == blockNumber) {
        return indexBlockNumber;
      } else if (indexBlockNumber < blockNumber) {
        lower = index;
      } else {
        upper = index - 1;
      }
    }

    return blockNumbers[lower];
  }
}
