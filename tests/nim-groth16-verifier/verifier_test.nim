import
  std/[unittest, strformat],
  std/os,
  ../../libs/nim/nim-groth16-verifier/verify

const root = staticExec("git rev-parse --show-toplevel")

suite "TestVerify":
  test "Verify circuit1k":
    check(mainVerify(root & "/tests/nim-groth16-verifier/go-verifier-data-files/circuit1k", 1))

  test "Verify circuit5k":
    check(mainVerify(root & "/tests/nim-groth16-verifier/go-verifier-data-files/circuit5k", 5))

  for i in 291..533:
    test fmt"Verify proof {i}":
      check(mainVerify(root & "/vendor/eth2-light-client-updates/mainnet/recursive-proofs", i))
