// SPDX-License-Identifier: MIT
pragma solidity 0.8.9;
import './Verifier.sol';

contract BLSVerify is Verifier {
  function verifySignature(
    uint256[2] memory a,
    uint256[2][2] memory b,
    uint256[2] memory c,
    bytes32 prev_header_hash,
    bytes32 next_header_hash
  ) internal view returns (bool) {
    uint256[4] memory input;

    uint256 prev_header_hash1 = (uint256(prev_header_hash) &
      (((1 << 253) - 1) << 3)) >> 3;

    uint256 prev_header_hash2 = (uint256(prev_header_hash) & ((1 << 3) - 1));

    input[0] = prev_header_hash1;
    input[1] = prev_header_hash2;

    uint256 next_header_hash1 = ((uint256(next_header_hash) &
      (((1 << 253) - 1) << 3)) >> 3);

    uint256 next_header_hash2 = (uint256(next_header_hash) & ((1 << 3) - 1));

    input[2] = next_header_hash1;
    input[3] = next_header_hash2;

    return verifyProof(a, b, c, input);
  }
}
