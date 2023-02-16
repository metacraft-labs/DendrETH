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

    uint256 commitment1 = reverseBits(
      (uint256(commitment) & (((1 << 253) - 1) << 3)) >> 3,
      253
    );

    uint256 commitment2 = reverseBits(
      (uint256(commitment) & ((1 << 3) - 1)),
      3
    );

    uint256[2] memory input;

    input[0] = prev_header_hash1;
    input[1] = prev_header_hash2;

    return verifyProof(a, b, c, input);
  }

  // TODO handle this in circuit
  function reverseBits(uint256 x, uint256 size) private pure returns (uint256) {
    require(size <= 256);

    uint256 result = 0;
    for (uint256 i = 0; i < size; i++) {
      result = (result << 1) | (x & 1);
      x = x >> 1;
    }

    return result;
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
