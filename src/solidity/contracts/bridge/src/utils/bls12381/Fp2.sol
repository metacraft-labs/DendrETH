// SPDX-License-Identifier: MIT
pragma solidity 0.8.9;

import './Fp.sol';

struct Fp2 {
  Fp c0;
  Fp c1;
}

library FP2 {
  using FP for Fp;

  function eq(Fp2 memory x, Fp2 memory y) internal pure returns (bool) {
    return (x.c0.eq(y.c0) && x.c1.eq(y.c1));
  }

  function is_zero(Fp2 memory x) internal pure returns (bool) {
    return x.c0.is_zero() && x.c1.is_zero();
  }

  // Note: Zcash uses (x_im, x_re)
  function serialize(Fp2 memory x) internal pure returns (bytes memory) {
    return abi.encodePacked(x.c1.serialize(), x.c0.serialize());
  }
}
