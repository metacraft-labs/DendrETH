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
  function get_validators_accumulator()
    external
    view
    override
    returns (bytes32 node)
  {
    uint256 size = validatorsCount;

    // Calculate the Merkle accumulator root
    for (
      uint256 height = 0;
      height < VALIDATOR_ACCUMULATOR_TREE_DEPTH;
      height++
    ) {
      // This if-else structure supports tree balancing
      // If size is odd, the new node will be hashed with the previous node on this level
      // If size is even, the new node will be hashed with a predefined zero hash
      if ((size & 1) == 1)
        node = sha256(abi.encodePacked(branch[height], node));
      else node = sha256(abi.encodePacked(node, zeroHashes[height]));

      size /= 2;
    }
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

    // Calculate hash tree root of deposit message
    bytes32 depositMessageRoot = sha256(
      abi.encodePacked(
        sha256(
          abi.encodePacked(
            sha256(abi.encodePacked(pubkey, bytes16(0))),
            withdrawalCredentials
          )
        ),
        sha256(
          abi.encodePacked(
            toLe64(uint64(msg.value / 1 gwei)),
            bytes24(0),
            bytes32(0)
          )
        )
      )
    );

    bytes memory depositIndex = IDeposit(depositAddress).get_deposit_count();
    emit Deposited(pubkey, depositIndex, signature, depositMessageRoot);

    // Create a node for the validator
    bytes32 node = sha256(
      abi.encodePacked(pubkey, depositIndex, depositMessageRoot, signature)
    );

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
        return;
      }
      node = sha256(abi.encodePacked(branch[height], node));
      size /= 2;
    }
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
