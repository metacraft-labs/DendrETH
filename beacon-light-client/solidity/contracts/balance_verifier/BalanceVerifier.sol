// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.20;

import {PlonkVerifier} from './verifier.sol';
import {IBalanceVerifier} from './interfaces/IBalanceVerifierDiva.sol';
import {ZeroAddressError} from '../Errors.sol';
import '@openzeppelin/contracts/access/Ownable.sol';

abstract contract BalanceVerifier is Ownable, IBalanceVerifier {
  /// @notice The genesis block timestamp.
  uint256 public immutable GENESIS_BLOCK_TIMESTAMP;

  /// @notice The length of the beacon roots ring buffer.
  uint256 internal constant BEACON_ROOTS_HISTORY_BUFFER_LENGTH = 8191;

  /// @notice The address of the beacon roots precompile.
  /// @dev https://eips.ethereum.org/EIPS/eip-4788
  address internal constant BEACON_ROOTS =
    0x000F3df6D732807Ef1319fB7B8bB8522d0Beac02;

  address internal verifier;

  /// @notice the verifierDigest of the plonky2 circuit
  uint256 public verifierDigest;

  constructor(
    uint256 _verifierDigest,
    uint256 genesisBlockTimestamp,
    address _verifier,
    address _owner
  ) Ownable(_owner) {
    if (_verifier == address(0)) {
      revert ZeroAddressError();
    }
    verifier = _verifier;
    verifierDigest = _verifierDigest;
    GENESIS_BLOCK_TIMESTAMP = genesisBlockTimestamp;
  }

  function setVerifier(
    address _verifier,
    uint256 newVerifierDigest
  ) external override onlyOwner {
    if (_verifier == address(0)) {
      revert ZeroAddressError();
    }
    verifier = _verifier;
    verifierDigest = newVerifierDigest;
  }

  /// @notice Verifies the proof and writes the data for given slot if valid
  /// @param proof the zk proof for total value locked
  /// @param publicInputs the public inputs for the proof
  function _verify(
    bytes calldata proof,
    uint256[] memory publicInputs
  ) internal {
    (bool success, bytes memory returnData) = verifier.call(
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
  ///      N, you use the timestamp of slot N+1. If N+1 is not available, you use the timestamp of slot N+2, and
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
