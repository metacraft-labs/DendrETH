// SPDX-License-Identifier: MIT
pragma solidity 0.8.9;

import './Verifier.sol';

contract LightClientUpdateVerifier is Groth16Verifier {
  error VerificationCallFailed();

  function verifyUpdate(
    uint256[2] memory a,
    uint256[2][2] memory b,
    uint256[2] memory c,
    bytes32 prevHeaderHash,
    bytes32 nextHeaderHash,
    uint256 nextHeaderSlot,
    bytes32 finalizedHeaderRoot,
    bytes32 executionStateRoot,
    bytes32 domain
  ) internal returns (bool) {
    bytes memory concatenated = abi.encodePacked(prevHeaderHash, nextHeaderHash, finalizedHeaderRoot, executionStateRoot, nextHeaderSlot, domain);
    bytes32 commitment = sha256(concatenated);

    uint256[2] memory input;

    input[0] = (uint256(commitment) & (((1 << 253) - 1) << 3)) >> 3;
    input[1] = (uint256(commitment) & ((1 << 3) - 1));

    (bool success, bytes memory returnData) = address(this).call(
      // Encode the call to the `verify` function with the public inputs
      abi.encodeWithSelector(Groth16Verifier.verifyProof.selector, a, b, c, input)
    );

    // Check if the call was successful
    if (!success) {
      revert VerificationCallFailed();
    }

    return abi.decode(returnData, (bool));
  }
}
