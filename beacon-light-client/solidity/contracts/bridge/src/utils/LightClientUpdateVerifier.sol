// SPDX-License-Identifier: MIT
pragma solidity 0.8.9;
import './Verifier.sol';

contract LightClientUpdateVerifier is Verifier {
  function verifyUpdate(
    uint256[2] memory a,
    uint256[2][2] memory b,
    uint256[2] memory c,
    bytes32 prev_header_hash,
    bytes32 next_header_hash,
    bytes32 finalized_header_root,
    bytes32 execution_state_root
  ) internal view returns (bool) {
    bytes32 commitment = hash(
      prev_header_hash,
      next_header_hash,
      finalized_header_root,
      execution_state_root
    );

    uint256[2] memory input;

    input[0] = (uint256(commitment) & (((1 << 253) - 1) << 3)) >> 3;
    input[1] = (uint256(commitment) & ((1 << 3) - 1));

    return verifyProof(a, b, c, input);
  }

  function hash(
    bytes32 a,
    bytes32 b,
    bytes32 c,
    bytes32 d
  ) private pure returns (bytes32) {
    bytes memory concatenated = abi.encodePacked(a, b, c, d);
    return sha256(concatenated);
  }
}
