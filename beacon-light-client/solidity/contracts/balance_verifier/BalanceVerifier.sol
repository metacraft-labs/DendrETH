// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.19;

import './verifier.sol';

contract BalanceVerifier is PlonkVerifier {
  uint256 public constant VERIFIER_DIGEST =
    12132998113779983235430917548537520464854579851393401583800381700464695543790;
  bytes32 public constant WITHDRAWAL_CREDENTIALS =
    0x01000000000000000000000015f4b914a0ccd14333d850ff311d6dafbfbaa32b;

  function verify(
    bytes calldata proof,
    bytes32 state_root,
    uint64 balance_sum,
    uint64 number_of_non_activated_validators,
    uint64 number_of_active_validators,
    uint64 number_of_exited_validators
  ) public {
    bytes memory concataneted = abi.encodePacked(
      state_root,
      WITHDRAWAL_CREDENTIALS,
      balance_sum,
      number_of_non_activated_validators,
      number_of_active_validators,
      number_of_exited_validators
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
  }
}
