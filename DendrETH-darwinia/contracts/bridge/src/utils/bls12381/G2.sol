// SPDX-License-Identifier: MIT
pragma solidity 0.7.6;

import './Fp2.sol';
import '../Bytes.sol';

struct G2Point {
  Fp2 x;
  Fp2 y;
}

library G2 {
  using FP for Fp;
  using FP2 for Fp2;
  using Bytes for bytes;

  uint8 private constant G2_ADD = 0x0D;
  uint8 private constant G2_MUL = 0x0E;
  uint8 private constant MAP_FP2_TO_G2 = 0x12;

  bytes1 private constant COMPRESION_FLAG = bytes1(0x80);
  bytes1 private constant INFINITY_FLAG = bytes1(0x40);
  bytes1 private constant Y_FLAG = bytes1(0x20);

  function eq(G2Point memory p, G2Point memory q) internal pure returns (bool) {
    return (p.x.eq(q.x) && p.y.eq(q.y));
  }

  function is_zero(G2Point memory p) internal pure returns (bool) {
    return p.x.is_zero() && p.y.is_zero();
  }

  function is_infinity(G2Point memory p) internal pure returns (bool) {
    return is_zero(p);
  }

  function add(G2Point memory p, G2Point memory q)
    internal
    view
    returns (G2Point memory)
  {
    uint256[16] memory input;
    input[0] = p.x.c0.a;
    input[1] = p.x.c0.b;
    input[2] = p.x.c1.a;
    input[3] = p.x.c1.b;
    input[4] = p.y.c0.a;
    input[5] = p.y.c0.b;
    input[6] = p.y.c1.a;
    input[7] = p.y.c1.b;
    input[8] = q.x.c0.a;
    input[9] = q.x.c0.b;
    input[10] = q.x.c1.a;
    input[11] = q.x.c1.b;
    input[12] = q.y.c0.a;
    input[13] = q.y.c0.b;
    input[14] = q.y.c1.a;
    input[15] = q.y.c1.b;
    uint256[8] memory output;

    assembly {
      if iszero(staticcall(4500, G2_ADD, input, 512, output, 256)) {
        returndatacopy(0, 0, returndatasize())
        revert(0, returndatasize())
      }
    }

    return from(output);
  }

  function mul(G2Point memory p, uint256 scalar)
    internal
    view
    returns (G2Point memory)
  {
    uint256[9] memory input;
    input[0] = p.x.c0.a;
    input[1] = p.x.c0.b;
    input[2] = p.x.c1.a;
    input[3] = p.x.c1.b;
    input[4] = p.y.c0.a;
    input[5] = p.y.c0.b;
    input[6] = p.y.c1.a;
    input[7] = p.y.c1.b;
    input[8] = scalar;
    uint256[8] memory output;

    assembly {
      if iszero(staticcall(55000, G2_MUL, input, 288, output, 256)) {
        returndatacopy(0, 0, returndatasize())
        revert(0, returndatasize())
      }
    }

    return from(output);
  }

  function map_to_curve(Fp2 memory f) internal view returns (G2Point memory) {
    uint256[4] memory input;
    input[0] = f.c0.a;
    input[1] = f.c0.b;
    input[2] = f.c1.a;
    input[3] = f.c1.b;
    uint256[8] memory output;

    assembly {
      if iszero(staticcall(110000, MAP_FP2_TO_G2, input, 128, output, 256)) {
        returndatacopy(0, 0, returndatasize())
        revert(0, returndatasize())
      }
    }

    return from(output);
  }

  function from(uint256[8] memory x) internal view returns (G2Point memory) {
    return
      G2Point(
        Fp2(Fp(x[0], x[1]), Fp(x[2], x[3])),
        Fp2(Fp(x[4], x[5]), Fp(x[6], x[7]))
      );
  }

  // Take a 192 byte array and convert to G2 point (x, y)
  function deserialize(bytes memory g2) internal pure returns (G2Point memory) {
    require(g2.length == 192, '!g2');
    bytes1 byt = g2[0];
    require(byt & COMPRESION_FLAG == 0, 'compressed');
    require(byt & INFINITY_FLAG == 0, 'infinity');
    require(byt & Y_FLAG == 0, 'y_flag');

    g2[0] = byt & 0x1f;

    // Convert from array to FP2
    Fp memory x_imaginary = Fp(
      g2.slice_to_uint(0, 16),
      g2.slice_to_uint(16, 48)
    );
    Fp memory x_real = Fp(g2.slice_to_uint(48, 64), g2.slice_to_uint(64, 96));
    Fp memory y_imaginary = Fp(
      g2.slice_to_uint(96, 112),
      g2.slice_to_uint(112, 144)
    );
    Fp memory y_real = Fp(
      g2.slice_to_uint(144, 160),
      g2.slice_to_uint(160, 192)
    );

    // Require elements less than field modulus
    require(
      x_imaginary.is_valid() &&
        x_real.is_valid() &&
        y_imaginary.is_valid() &&
        y_real.is_valid(),
      '!pnt'
    );

    Fp2 memory x = Fp2(x_real, x_imaginary);
    Fp2 memory y = Fp2(y_real, y_imaginary);

    G2Point memory p = G2Point(x, y);
    require(!is_infinity(p), 'infinity');
    return p;
  }

  // Take a G2 point (x, y) and compress it to a 96 byte array as the x-coordinate.
  function serialize(G2Point memory g2) internal pure returns (bytes memory r) {
    if (is_infinity(g2)) {
      r = new bytes(96);
      r[0] = bytes1(0xc0);
    } else {
      // Convert x-coordinate to bytes
      r = g2.x.serialize();

      // Record the leftmost bit of y_im to the a_flag1
      // If y_im happens to be zero, then use the bit of y_re
      // y_flag = (y_im * 2) // q if y_im > 0 else (y_re * 2) // q
      Fp memory q = FP.q();
      Fp memory y_re = g2.y.c0;
      Fp memory y_im = g2.y.c1;

      bool y_flag = y_im.is_zero()
        ? y_re.add(y_re).gt(q)
        : y_im.add(y_im).gt(q);
      if (y_flag) {
        r[0] = r[0] | Y_FLAG;
      }
      r[0] = r[0] | COMPRESION_FLAG;
    }
  }
}
