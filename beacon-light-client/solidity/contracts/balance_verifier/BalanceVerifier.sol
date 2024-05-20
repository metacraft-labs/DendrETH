// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import {PlonkVerifier} from './verifier.sol';
import {IBalanceVerifier} from './interfaces/IBalanceVerifierDiva.sol';

abstract contract BalanceVerifier is PlonkVerifier, IBalanceVerifier {
  /// @notice the verifierDigest of the plonky2 circuit
  uint256 public immutable VERIFIER_DIGEST;

  /// @notice lido validators withdrawal credentials
  bytes32 public immutable WITHDRAWAL_CREDENTIALS;

  /// @notice The genesis block timestamp.
  uint256 public immutable GENESIS_BLOCK_TIMESTAMP;

  /// @notice The length of the beacon roots ring buffer.
  uint256 internal constant BEACON_ROOTS_HISTORY_BUFFER_LENGTH = 8191;

  /// @notice The address of the beacon roots precompile.
  /// @dev https://eips.ethereum.org/EIPS/eip-4788
  address internal constant BEACON_ROOTS =
    0x000F3df6D732807Ef1319fB7B8bB8522d0Beac02;

  constructor(
    uint256 verifierDigest,
    bytes32 withdrawalCredentials,
    uint256 genesisBlockTimestamp
  ) {
    VERIFIER_DIGEST = verifierDigest;
    WITHDRAWAL_CREDENTIALS = withdrawalCredentials;
    GENESIS_BLOCK_TIMESTAMP = genesisBlockTimestamp;
  }

  /// @notice Verifies the proof and writes the data for given slot if valid
  /// @param proof the zk proof for total value locked
  /// @param slot the slot for which the proof is ran
  /// @param balanceSum the sum of the balances of all validators with withdrawal credentials equal to WITHDRAWAL_CREDENTIALS
  /// @param _numberOfNonActivatedValidators number of validators yet to be activated
  /// @param _numberOfActiveValidators number of active validators
  /// @param _numberOfExitedValidators number of exited validators
  function _verify(
    bytes calldata proof,
    uint256 slot,
    uint64 balanceSum,
    uint64 _numberOfNonActivatedValidators,
    uint64 _numberOfActiveValidators,
    uint64 _numberOfExitedValidators,
    uint64 _numberOfSlashedValidators
  ) internal {
    uint256[] memory publicInputs = new uint256[](2);
    publicInputs[0] = VERIFIER_DIGEST;
    publicInputs[1] = (uint256(
      sha256(
        abi.encodePacked(
          _findBlockRoot(slot),
          WITHDRAWAL_CREDENTIALS,
          balanceSum,
          _numberOfNonActivatedValidators,
          _numberOfActiveValidators,
          _numberOfExitedValidators,
          _numberOfSlashedValidators
        )
      )
    ) & ((1 << 253) - 1));

    // Make the call using `address(this).call`
    (bool success, bytes memory returnData) = address(this).call(
      // Encode the call to the `verify` function with the public inputs
      abi.encodeWithSelector(PlonkVerifier.Verify.selector, proof, publicInputs)
    );

    // Check if the call was successful
    if (!success) {
      revert VerificationCallFailed();
    }

    bool verificationResult = abi.decode(returnData, (bool));

    if (!verificationResult) {
      revert VerificationFailed();
    }
  }

  /// @notice Attempts to find the block root for the given slot.
  /// @param _slot The slot to get the block root for.
  /// @return blockRoot The beacon block root of the given slot.
  /// @dev BEACON_ROOTS returns a block root for a given parent block's timestamp. To get the block root for slot
  ///      N, you use the timestamp of slot N+1. If N+1 is not avaliable, you use the timestamp of slot N+2, and
  //       so on.
  function _findBlockRoot(uint256 _slot) internal view returns (bytes32) {
    uint256 currBlockTimestamp = GENESIS_BLOCK_TIMESTAMP + ((_slot + 1) * 12);

    if (
      currBlockTimestamp <=
      block.timestamp - (BEACON_ROOTS_HISTORY_BUFFER_LENGTH * 12)
    ) {
      revert BeaconRootOutOfRange();
    }

    while (currBlockTimestamp <= block.timestamp) {
      (bool success, bytes memory result) = BEACON_ROOTS.staticcall(
        abi.encode(currBlockTimestamp)
      );
      if (success && result.length > 0) {
        return abi.decode(result, (bytes32));
      }

      unchecked {
        currBlockTimestamp += 12;
      }
    }

    revert NoBlockRootFound();
  }
}
