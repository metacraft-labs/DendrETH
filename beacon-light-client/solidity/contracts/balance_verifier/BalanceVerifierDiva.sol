// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import './verifier.sol';
import './DivaZKOracle.sol';

contract BalanceVerifier is PlonkVerifier, IZKOracle {
  /// @notice the verifier_digest of the plonky2 circuit
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

  /// @dev Beacon root out of range
  error BeaconRootOutOfRange();

  /// @dev No block root is found using the beacon roots precompile.
  error NoBlockRootFound();

  /// @dev Verification call failed
  error VerificationCallFailed();

  /// @dev Verification failed
  error VerificationFailed();

  mapping(uint256 => Report) reports;

  constructor(
    uint256 verifier_digest,
    bytes32 withdrawal_credentials,
    uint256 genesis_block_timestamp
  ) {
    VERIFIER_DIGEST = verifier_digest;
    WITHDRAWAL_CREDENTIALS = withdrawal_credentials;
    GENESIS_BLOCK_TIMESTAMP = genesis_block_timestamp;
  }

  /// @notice Verifies the proof and writes the data for given slot if valid
  /// @param proof the zk proof for total value locked
  /// @param refSlot the slot for which the proof is ran
  /// @param balanceSum the sum of the balances of all validators with withdrawal credentials equal to WITHDRAWAL_CREDENTIALS
  /// @param _numberOfNonActivatedValidators number of validators yet to be activated
  /// @param _numberOfActiveValidators number of active validators
  /// @param _numberOfExitedValidators number of exited validators
  function verify(
    bytes calldata proof,
    uint256 slot,
    uint64 balanceSum,
    uint64 _numberOfNonActivatedValidators,
    uint64 _numberOfActiveValidators,
    uint64 _numberOfExitedValidators,
    uint64 _numberOfSlashedValidators
  ) public {
    bytes32 blockRoot = findBlockRoot(slot);

    bytes memory concataneted = abi.encodePacked(
      blockRoot,
      WITHDRAWAL_CREDENTIALS,
      balanceSum,
      _numberOfNonActivatedValidators,
      _numberOfActiveValidators,
      _numberOfExitedValidators,
      _numberOfSlashedValidators
    );

    bytes32 commitment = sha256(concataneted);

    uint256[] memory publicInputs = new uint256[](2);
    publicInputs[0] = VERIFIER_DIGEST;
    publicInputs[1] = (uint256(commitment) & ((1 << 253) - 1));

    // Encode the call to the `verify` function with the public inputs
    bytes memory data = abi.encodeWithSelector(
      PlonkVerifier.Verify.selector,
      proof,
      publicInputs
    );

    // Make the call using `address(this).call`
    (bool success, bytes memory returnData) = address(this).call(data);

    // Check if the call was successful
    if (!success) {
      revert VerificationCallFailed();
    }

    bool verificationResult = abi.decode(returnData, (bool));

    if (!verificationResult) {
      revert VerificationFailed();
    }

    uint64 numValidators = _numberOfActiveValidators +
      _numberOfExitedValidators;

    reports[slot] = Report({
      present: true,
      cBalanceGwei: balanceSum,
      numValidators: numValidators,
      exitedValidators: _numberOfExitedValidators,
      slashedValidators: _numberOfSlashedValidators
    });

    emit Report(
      slot,
      balanceSum,
      numValidators,
      _numberOfExitedValidators,
      _numberOfSlashedValidators
    );
  }

  function getReport(
    uint256 slot
  )
    external
    view
    override
    returns (
      bool success,
      uint256 clBalanceGwei,
      uint256 numValidators,
      uint256 exitedValidators
    )
  {
    Report memory report = reports[slot];

    return (
      report.present,
      report.cBalanceGwei,
      report.numValidators,
      report.exitedValidators
    );
  }

  /// @notice Attempts to find the block root for the given slot.
  /// @param _slot The slot to get the block root for.
  /// @return blockRoot The beacon block root of the given slot.
  /// @dev BEACON_ROOTS returns a block root for a given parent block's timestamp. To get the block root for slot
  ///      N, you use the timestamp of slot N+1. If N+1 is not avaliable, you use the timestamp of slot N+2, and
  //       so on.
  function findBlockRoot(
    uint256 _slot
  ) public view returns (bytes32 blockRoot) {
    uint256 currBlockTimestamp = GENESIS_BLOCK_TIMESTAMP + ((_slot + 1) * 12);

    uint256 earliestBlockTimestamp = block.timestamp -
      (BEACON_ROOTS_HISTORY_BUFFER_LENGTH * 12);
    if (currBlockTimestamp <= earliestBlockTimestamp) {
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
