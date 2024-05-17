// SPDX-License-Identifier: MIT
pragma solidity 0.8.18;

import {IDeposit} from './interfaces/IDeposit.sol';
import {IValidatorsAccumulator} from './interfaces/IValidatorsAccumulator.sol';

contract ValidatorsAccumulator is IValidatorsAccumulator {
  // The depth of the validator accumulator tree
  uint8 internal constant VALIDATOR_ACCUMULATOR_TREE_DEPTH = 32;
  address internal immutable depositAddress;

  // An array to hold the branch hashes for the Merkle tree
  bytes32[VALIDATOR_ACCUMULATOR_TREE_DEPTH] internal branch;
  bytes32[VALIDATOR_ACCUMULATOR_TREE_DEPTH] internal zero_hashes;
  // A counter for the total number of validators
  uint256 internal validators_count;

  constructor(address _depositAddress) {
    depositAddress = _depositAddress;

    // Compute hashes in empty Merkle tree
    for (
      uint256 height = 0;
      height < VALIDATOR_ACCUMULATOR_TREE_DEPTH - 1;
      height++
    )
      zero_hashes[height + 1] = sha256(
        abi.encodePacked(zero_hashes[height], zero_hashes[height])
      );
  }

  // Function to calculate and return the Merkle accumulator root of the validators
  function get_validators_accumulator()
    external
    view
    override
    returns (bytes32 node)
  {
    uint256 size = validators_count;

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
      else node = sha256(abi.encodePacked(node, zero_hashes[height]));

      size /= 2;
    }
  }

  // Function to handle deposits from validators
  // TODO: Maybe we can construct the accumulator using posiedon hash directly
  function deposit(
    bytes calldata pubkey,
    bytes calldata withdrawal_credentials,
    bytes calldata signature,
    bytes32 deposit_data_root
  ) external payable override {
    // Perform the deposit using the DepositContract

    IDeposit(depositAddress).deposit{value: msg.value}(
      pubkey,
      withdrawal_credentials,
      signature,
      deposit_data_root
    );

    validators_count += 1;

    // Create a node for the validator
    bytes32 node = sha256(
      abi.encodePacked(
        pubkey,
        IDeposit(depositAddress).get_deposit_count() // Get the deposit count and increase the validator count
      )
    );

    // Insert the node into the Merkle accumulator tree
    uint256 size = validators_count;
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

    emit Deposited(
      pubkey,
      withdrawal_credentials,
      signature,
      deposit_data_root
    );
  }
}
