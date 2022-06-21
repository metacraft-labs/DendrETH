import ../../../nimbus-eth2/vendor/nimcrypto/nimcrypto/hash

type
  Eth2Digest* = MDigest[32 * 8] ## `hash32` from spec
