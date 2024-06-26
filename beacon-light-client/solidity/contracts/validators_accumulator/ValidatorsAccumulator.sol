// SPDX-License-Identifier: MIT
pragma solidity ^0.8.18;

import {IDeposit} from './interfaces/IDeposit.sol';
import {IValidatorsAccumulator} from './interfaces/IValidatorsAccumulator.sol';

contract ValidatorsAccumulator is IValidatorsAccumulator {
  // The depth of the validator accumulator tree
  uint8 internal constant VALIDATOR_ACCUMULATOR_TREE_DEPTH = 32;
  address internal immutable depositAddress;

  // An array to hold the branch hashes for the Merkle tree
  bytes32[VALIDATOR_ACCUMULATOR_TREE_DEPTH] internal branch;
  bytes32[VALIDATOR_ACCUMULATOR_TREE_DEPTH] internal zeroHashes;

  // A counter for the total number of validators
  uint256 internal validatorsCount;
  // Start index of validators map
  uint256 internal startIndex;

  mapping(uint256 => DepositData) internal snapshots;

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
  function getValidatorsAccumulator()
    external
    view
    override
    returns (bytes32 node)
  {
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
    uint256 size = validatorsCount;
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

    snapshots[validatorsCount - 1] = DepositData({
      blockNumber: block.number,
      accumulator: _getRoot(validatorsCount)
    });
  }

  function findAccumulatorByBlock(
    uint256 blockNumber
  ) external view override returns (uint256, bytes32) {
    if (validatorsCount == 0) {
      return (0, zeroHashes[VALIDATOR_ACCUMULATOR_TREE_DEPTH - 1]);
    }

    uint256 index = _binarySearchBlock(blockNumber);

    DepositData memory snapshot = snapshots[index];

    if (snapshot.blockNumber > blockNumber) {
      return (0, zeroHashes[VALIDATOR_ACCUMULATOR_TREE_DEPTH - 1]);
    }

    return (index + 1, snapshot.accumulator);
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
    uint256 lower = startIndex;
    uint256 upper = validatorsCount - 1;

    if (snapshots[upper].blockNumber <= blockNumber) {
      return upper;
    }

    if (snapshots[lower].blockNumber > blockNumber) {
      return 0;
    }

    while (upper > lower) {
      uint256 index = upper - (upper - lower) / 2; // ceil, avoiding overflow
      DepositData memory snapshot = snapshots[index];
      if (snapshot.blockNumber == blockNumber) {
        return index;
      } else if (snapshot.blockNumber < blockNumber) {
        lower = index;
      } else {
        upper = index - 1;
      }
    }

    return lower;
  }

  function toLe64(uint64 value) internal pure returns (bytes memory ret) {
    ret = new bytes(8);
    bytes8 bytesValue = bytes8(value);
    // Byteswapping during copying to bytes.
    ret[0] = bytesValue[7];
    ret[1] = bytesValue[6];
    ret[2] = bytesValue[5];
    ret[3] = bytesValue[4];
    ret[4] = bytesValue[3];
    ret[5] = bytesValue[2];
    ret[6] = bytesValue[1];
    ret[7] = bytesValue[0];
  }
}
