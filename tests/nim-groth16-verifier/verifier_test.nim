import
  std/[unittest, strformat]

import
  ../../libs/nim/nim-groth16-verifier/verify

const root = staticExec("git rev-parse --show-toplevel")

suite "TestVerify":
  test "Verify circuit1k":
    check(verify(root & "/tests/nim-groth16-verifier/go-verifier-data-files/circuit1k", 1))

  test "Verify circuit5k":
    check(verify(root & "/tests/nim-groth16-verifier/go-verifier-data-files/circuit5k", 5))

  for i in 291..533:
    test fmt"Verify proof {i}":
      check(verify(root & "/vendor/eth2-light-client-updates/mainnet/proofs", i))

  for i in 291..533:
    test fmt"Verify recursive-proof {i}":
      check(verify(root & "/vendor/eth2-light-client-updates/mainnet/recursive-proofs", i))
