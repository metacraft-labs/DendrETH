// SPDX-License-Identifier: MIT
pragma solidity 0.8.9;

import './Verifier.sol';

contract LightClientUpdateVerifier is Verifier {
  function verifyUpdate(
    uint256[2] memory a,
    uint256[2][2] memory b,
    uint256[2] memory c,
    bytes32 prevHeaderHash,
    bytes32 nextHeaderHash,
    uint256 nextHeaderSlot,
    bytes32 finalizedHeaderRoot,
    bytes32 executionStateRoot
  ) internal view returns (bool) {
    bytes32 commitment = hash(
      prevHeaderHash,
      nextHeaderHash,
      nextHeaderSlot,
      finalizedHeaderRoot,
      executionStateRoot
    );

    uint256[2] memory input;

    input[0] = (uint256(commitment) & (((1 << 253) - 1) << 3)) >> 3;
    input[1] = (uint256(commitment) & ((1 << 3) - 1));

    return verifyProof(a, b, c, input);
  }

  function hash(
    bytes32 a,
    bytes32 b,
    uint256 c,
    bytes32 d,
    bytes32 e
  ) private pure returns (bytes32) {
    bytes memory concatenated = abi.encodePacked(a, b, c, d, e);
    return sha256(concatenated);
  }
}
