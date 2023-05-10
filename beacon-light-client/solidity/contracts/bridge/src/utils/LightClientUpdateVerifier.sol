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
    bytes32 executionStateRoot,
    bytes32 domain
  ) internal view returns (bool) {
    bytes memory concatenated = abi.encodePacked(prevHeaderHash, nextHeaderHash, finalizedHeaderRoot, executionStateRoot, nextHeaderSlot, domain);
    bytes32 commitment = sha256(concatenated);

    uint256[2] memory input;

    input[0] = (uint256(commitment) & (((1 << 253) - 1) << 3)) >> 3;
    input[1] = (uint256(commitment) & ((1 << 3) - 1));

    return verifyProof(a, b, c, input);
  }
}
