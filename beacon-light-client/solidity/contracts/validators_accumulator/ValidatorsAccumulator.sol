// SPDX-License-Identifier: MIT
pragma solidity 0.8.18;

// This interface is designed to be compatible with the Vyper version.
/// @notice This is the Ethereum 2.0 deposit contract interface.
/// For more information see the Phase 0 specification under https://github.com/ethereum/eth2.0-specs
interface IDepositContract {
  /// @notice A processed deposit event.
  event DepositEvent(
    bytes pubkey,
    bytes withdrawal_credentials,
    bytes amount,
    bytes signature,
    bytes index
  );

  /// @notice Submit a Phase 0 DepositData object.
  /// @param pubkey A BLS12-381 public key.
  /// @param withdrawal_credentials Commitment to a public key for withdrawals.
  /// @param signature A BLS12-381 signature.
  /// @param deposit_data_root The SHA-256 hash of the SSZ-encoded DepositData object.
  /// Used as a protection against malformed input.
  function deposit(
    bytes calldata pubkey,
    bytes calldata withdrawal_credentials,
    bytes calldata signature,
    bytes32 deposit_data_root
  ) external payable;

  /// @notice Query the current deposit root hash.
  /// @return The deposit root hash.
  function get_deposit_root() external view returns (bytes32);

  /// @notice Query the current deposit count.
  /// @return The deposit count encoded as a little endian 64-bit number.
  function get_deposit_count() external view returns (bytes memory);
}

contract ValidatorsAccumulator {
  address depositAddress;

  // The depth of the validator accumulator tree
  uint constant VALIDATOR_ACCUMULATOR_TREE_DEPTH = 32;

  // An array to hold the branch hashes for the Merkle tree
  bytes32[VALIDATOR_ACCUMULATOR_TREE_DEPTH] branch;

  bytes32[VALIDATOR_ACCUMULATOR_TREE_DEPTH] zero_hashes;

  // A counter for the total number of validators
  uint validators_count;

  constructor(address _depositAddress) {
    depositAddress = _depositAddress;

    // Compute hashes in empty Merkle tree
    for (
      uint height = 0;
      height < VALIDATOR_ACCUMULATOR_TREE_DEPTH - 1;
      height++
    )
      zero_hashes[height + 1] = sha256(
        abi.encodePacked(zero_hashes[height], zero_hashes[height])
      );
  }

  // Function to calculate and return the Merkle accumulator root of the validators
  function get_validators_accumulator() external view returns (bytes32) {
    bytes32 node;
    uint size = validators_count;

    // Calculate the Merkle accumulator root
    for (uint height = 0; height < VALIDATOR_ACCUMULATOR_TREE_DEPTH; height++) {
      // This if-else structure supports tree balancing
      // If size is odd, the new node will be hashed with the previous node on this level
      // If size is even, the new node will be hashed with a predefined zero hash
      if ((size & 1) == 1)
        node = sha256(abi.encodePacked(branch[height], node));
      else node = sha256(abi.encodePacked(node, zero_hashes[height]));

      size /= 2;
    }

    return node;
  }

  // Function to handle deposits from validators
  // TODO: Maybe we can construct the accumulator using posiedon hash directly
  function deposit(
    bytes calldata pubkey,
    bytes calldata withdrawal_credentials,
    bytes calldata signature,
    bytes32 deposit_data_root
  ) public payable {
    // Perform the deposit using the DepositContract

    IDepositContract(depositAddress).deposit{value: msg.value}(
      pubkey,
      withdrawal_credentials,
      signature,
      deposit_data_root
    );

    // Get the deposit count and increase the validator count
    bytes memory deposit_index = IDepositContract(depositAddress)
      .get_deposit_count();
    validators_count += 1;

    // Create a node for the validator
    bytes32 node = sha256(abi.encodePacked(pubkey, deposit_index));

    // Insert the node into the Merkle accumulator tree
    uint size = validators_count;
    for (uint height = 0; height < VALIDATOR_ACCUMULATOR_TREE_DEPTH; height++) {
      if ((size & 1) == 1) {
        branch[height] = node;
        return;
      }
      node = sha256(abi.encodePacked(branch[height], node));
      size /= 2;
    }
  }
}
