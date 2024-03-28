// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import './verifier.sol';
import './LidoZKOracle.sol';

uint256 constant BUFER_SIZE = 32;

contract BalanceVerifier is PlonkVerifier, LidoZKOracle {
  /// @notice The address of the beacon roots precompile.
  /// @dev https://eips.ethereum.org/EIPS/eip-4788
  address internal constant BEACON_ROOTS =
    0x000F3df6D732807Ef1319fB7B8bB8522d0Beac02;

  /// @notice the verifier_digest of the plonky2 circuit
  uint256 public immutable VERIFIER_DIGEST;

  /// @notice lido validators withdrawal credentials
  bytes32 public immutable WITHDRAWAL_CREDENTIALS;

  /// @notice The genesis block timestamp.
  uint256 public immutable GENESIS_BLOCK_TIMESTAMP;

  /// @notice The length of the beacon roots ring buffer.
  uint256 internal constant BEACON_ROOTS_HISTORY_BUFFER_LENGTH = 8191;

  /// @dev Beacon root out of range
  error BeaconRootOutOfRange();

  /// @dev No block root is found using the beacon roots precompile.
  error NoBlockRootFound();

  /// @notice The ring buffer of the reports.
  Report[BUFER_SIZE] reports;

  /// @notice The current index in the ring buffer
  uint256 public currentIndex;

  constructor(
    uint256 verifier_digest,
    bytes32 withdrawal_credentials,
    uint256 genesis_block_timestamp
  ) {
    currentIndex = 0;
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
    uint64 refSlot,
    uint64 balanceSum,
    uint64 _numberOfNonActivatedValidators,
    uint64 _numberOfActiveValidators,
    uint64 _numberOfExitedValidators
  ) public {
    bytes32 blockRoot = findBlockRoot(refSlot);

    bytes memory concataneted = abi.encodePacked(
      blockRoot,
      WITHDRAWAL_CREDENTIALS,
      balanceSum,
      _numberOfNonActivatedValidators,
      _numberOfActiveValidators,
      _numberOfExitedValidators
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
    require(success, 'Verify function call failed');

    bool verificationResult = abi.decode(returnData, (bool));

    require(verificationResult, 'Verification failed');

    currentIndex++;
    reports[currentIndex % BUFER_SIZE] = Report({
      slot: refSlot,
      cBalanceGwei: balanceSum,
      numValidators: _numberOfNonActivatedValidators +
        _numberOfActiveValidators +
        _numberOfExitedValidators,
      exitedValidators: _numberOfExitedValidators
    });
  }

  function getReport(
    uint256 refSlot
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
    uint256 i = 0;
    uint256 checkIndex = currentIndex % BUFER_SIZE;
    do {
      Report memory report = reports[checkIndex];

      if (report.slot == refSlot) {
        return (
          true,
          report.cBalanceGwei,
          report.numValidators,
          report.exitedValidators
        );
      }

      i++;
      checkIndex--;
    } while (i < BUFER_SIZE);

    return (false, 0, 0, 0);
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
