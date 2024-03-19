// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import './verifier.sol';
import './LidoZKOracle.sol';

contract BalanceVerifier is PlonkVerifier, LidoZKOracle {
  uint256 public immutable VERIFIER_DIGEST;
  bytes32 public immutable WITHDRAWAL_CREDENTIALS;

  mapping(uint256 => bytes32) public stateRoots;
  mapping(uint256 => uint256) public balanceSums;
  mapping(uint256 => uint256) public numberOfNonActivatedValidators;
  mapping(uint256 => uint256) public numberOfActiveValidators;
  mapping(uint256 => uint256) public numberOfExitedValidators;

  constructor(uint256 verifier_digest, bytes32 withdrawal_credentials) {
    VERIFIER_DIGEST = verifier_digest;
    WITHDRAWAL_CREDENTIALS = withdrawal_credentials;
  }

  function verify(
    bytes calldata proof,
    uint256 refSlot,
    bytes32 stateRoot,
    uint64 balanceSum,
    uint64 _numberOfNonActivatedValidators,
    uint64 _numberOfActiveValidators,
    uint64 _numberOfExitedValidators
  ) public {
    bytes memory concataneted = abi.encodePacked(
      stateRoot,
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

    stateRoots[refSlot] = stateRoot;
    balanceSums[refSlot] = balanceSum;
    numberOfNonActivatedValidators[refSlot] = _numberOfNonActivatedValidators;
    numberOfActiveValidators[refSlot] = _numberOfActiveValidators;
    numberOfExitedValidators[refSlot] = _numberOfExitedValidators;
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
    return (
      true,
      balanceSums[refSlot],
      numberOfActiveValidators[refSlot],
      numberOfExitedValidators[refSlot]
    );
  }
}
