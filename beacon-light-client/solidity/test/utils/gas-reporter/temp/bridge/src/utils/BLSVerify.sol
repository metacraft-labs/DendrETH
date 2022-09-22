// SPDX-License-Identifier: MIT
pragma solidity 0.8.9;
import './Verifier.sol';
import "hardhat/console.sol";
contract BLSVerify is Verifier {
  struct Fp {
    uint256 a;
    uint256 b;
  }

  uint8 constant MOD_EXP_PRECOMPILE_ADDRESS = 0x5;
  string constant BLS_SIG_DST = 'BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_POP_+';

  // Reduce the number encoded as the big-endian slice of data[start:end] modulo the BLS12-381 field modulus.
  // Copying of the base is cribbed from the following:
  // https://github.com/ethereum/solidity-examples/blob/f44fe3b3b4cca94afe9c2a2d5b7840ff0fafb72e/src/unsafe/Memory.sol#L57-L74
  function reduceModulo(
    bytes memory data,
    uint256 start,
    uint256 end
  ) private view returns (bytes memory) {
    uint256 length = end - start;
    assert(length <= data.length);

    bytes memory result = new bytes(48);

    bool success;
    assembly {
      let p := mload(0x40)
      // length of base
      mstore(p, length)
      // length of exponent
      mstore(add(p, 0x20), 0x20)
      // length of modulus
      mstore(add(p, 0x40), 48)
      // base
      // first, copy slice by chunks of EVM words
      let ctr := length
      let src := add(add(data, 0x20), start)
      let dst := add(p, 0x60)
      for {

      } or(gt(ctr, 0x20), eq(ctr, 0x20)) {
        ctr := sub(ctr, 0x20)
      } {
        mstore(dst, mload(src))
        dst := add(dst, 0x20)
        src := add(src, 0x20)
      }
      // next, copy remaining bytes in last partial word
      let mask := sub(exp(256, sub(0x20, ctr)), 1)
      let srcpart := and(mload(src), not(mask))
      let destpart := and(mload(dst), mask)
      mstore(dst, or(destpart, srcpart))
      // exponent
      mstore(add(p, add(0x60, length)), 1)
      // modulus
      let modulusAddr := add(p, add(0x60, add(0x10, length)))
      mstore(
        modulusAddr,
        or(mload(modulusAddr), 0x1a0111ea397fe69a4b1ba7b6434bacd7)
      ) // pt 1
      mstore(
        add(p, add(0x90, length)),
        0x64774b84f38512bf6730d2a0f6b0f6241eabfffeb153ffffb9feffffffffaaab
      ) // pt 2
      success := staticcall(
        sub(gas(), 2000),
        MOD_EXP_PRECOMPILE_ADDRESS,
        p,
        add(0xB0, length),
        add(result, 0x20),
        48
      )
      // Use "invalid" to make gas estimation work
      switch success
      case 0 {
        invalid()
      }
    }
    require(success, 'call to modular exponentiation precompile failed');
    return result;
  }

  function sliceToUint(
    bytes memory data,
    uint256 start,
    uint256 end
  ) private pure returns (uint256 result) {
    uint256 length = end - start;
    assert(length <= 32);

    for (uint256 i; i < length; ) {
      bytes1 b = data[start + i];
      result = result + (uint8(b) * 2**(8 * (length - i - 1)));
      unchecked {
        ++i;
      }
    }
  }

  function convertSliceToFp(
    bytes memory data,
    uint256 start,
    uint256 end
  ) private view returns (Fp memory) {
    bytes memory fieldElement = reduceModulo(data, start, end);
    uint256 a = sliceToUint(fieldElement, 0, 16);
    uint256 b = sliceToUint(fieldElement, 16, 48);
    return Fp(a, b);
  }

  function expandMessage(bytes32 message) private pure returns (bytes memory) {
    bytes memory b0Input = new bytes(143); //gas-reporter #114
    for (uint256 i; i < 32; ) {
      b0Input[i + 64] = message[i]; //gas-reporter #116
      unchecked {
        ++i; //gas-reporter #118
      }
    }
    b0Input[96] = 0x01; //gas-reporter #121
    for (uint256 i; i < 44; ) {
      b0Input[i + 99] = bytes(BLS_SIG_DST)[i]; //gas-reporter #123
      unchecked {
        ++i; //gas-reporter #125
      }
    }

    bytes32 b0 = sha256(abi.encodePacked(b0Input)); //gas-reporter #129

    bytes memory output = new bytes(256); //gas-reporter #131
    bytes32 chunk = sha256(
      abi.encodePacked(b0, bytes1(0x01), bytes(BLS_SIG_DST))
    ); //gas-reporter #134
    assembly {
      mstore(add(output, 0x20), chunk)
    }
 //gas-reporter #138
    for (uint256 i = 2; i < 9; ) {
      bytes32 input; //gas-reporter #140
      assembly {
        input := xor(b0, mload(add(output, add(0x20, mul(0x20, sub(i, 2))))))
      }
      chunk = sha256(
        abi.encodePacked(input, bytes1(uint8(i)), bytes(BLS_SIG_DST))
      ); //gas-reporter #146
      assembly {
        mstore(add(output, add(0x20, mul(0x20, sub(i, 1)))), chunk)
      } //gas-reporter #149
      unchecked {
        ++i; //gas-reporter #151
      }
    }

    return output;
  }

  function FpToArray55_7(Fp memory fp) public pure returns (uint256[7] memory) {
    uint256[7] memory result; //gas-reporter #159
    uint256 mask = ((1 << 55) - 1); //gas-reporter #160
    result[0] = (fp.b & (mask << (55 * 0))) >> (55 * 0); //gas-reporter #161
    result[1] = (fp.b & (mask << (55 * 1))) >> (55 * 1); //gas-reporter #162
    result[2] = (fp.b & (mask << (55 * 2))) >> (55 * 2); //gas-reporter #163
    result[3] = (fp.b & (mask << (55 * 3))) >> (55 * 3); //gas-reporter #164
    result[4] = (fp.b & (mask << (55 * 4))) >> (55 * 4); //gas-reporter #165
    uint256 newMask = (1 << 19) - 1; //gas-reporter #166
    result[4] = result[4] | ((fp.a & newMask) << 36); //gas-reporter #167
    result[5] = (fp.a & (mask << 19)) >> 19; //gas-reporter #168
    result[6] = (fp.a & (mask << (55 + 19))) >> (55 + 19); //gas-reporter #169

    return result;
  }

  function hashToField(bytes32 message)
    public
    view
    returns (uint256[7][2][2] memory result)
  {
    bytes memory some_bytes = expandMessage(message); //gas-reporter #179
    result[0][0] = FpToArray55_7(convertSliceToFp(some_bytes, 0, 64)); //gas-reporter #180
    result[0][1] = FpToArray55_7(convertSliceToFp(some_bytes, 64, 128)); //gas-reporter #181
    result[1][0] = FpToArray55_7(convertSliceToFp(some_bytes, 128, 192)); //gas-reporter #182
    result[1][1] = FpToArray55_7(convertSliceToFp(some_bytes, 192, 256)); //gas-reporter #183
  }

  function verifySignature(
    uint256[2] memory a,
    uint256[2][2] memory b,
    uint256[2] memory c,
    bytes32 signing_root,
    bytes32 sync_committee
  ) internal view returns (bool) {
    uint256[30] memory input; //gas-reporter #193

    uint256 sync_committee1 = (uint256(sync_committee) & ((1 << 3) - 1)); //gas-reporter #195

    uint256 sync_commitee2 = (uint256(sync_committee) &
      (((1 << 253) - 1) << 3)) >> 3; //gas-reporter #198

    input[0] = sync_committee1; //gas-reporter #200
    input[1] = sync_commitee2; //gas-reporter #201

    uint256[7][2][2] memory hashMessage = hashToField(signing_root); //gas-reporter #203
 //gas-reporter #204
    for (uint256 i = 0; i < 2; i++) { //gas-reporter #205
      for (uint256 j = 0; j < 2; j++) { //gas-reporter #206
        for (uint256 k = 0; k < 7; k++) {
          input[i * 14 + j * 7 + k + 2] = hashMessage[i][j][k]; //gas-reporter #208
        }
      }
    }

    return verifyProof(a, b, c, input);
  }
}