import
  std/unittest,
  # ../../libs/nim/verify-utils/verify_given_proof_constantine,
  ../../libs/nim/verify-utils/verify_given_proof

suite "description for this stuff":
  setup:
    let pathToKey = "vendor/eth2-light-client-updates/prater/capella-updates-94/vk.json"
    let pathToProof = "vendor/eth2-light-client-updates/prater/capella-updates-94/proof_5609044_5609069.json"
    let pathToLastUpdate = "vendor/eth2-light-client-updates/prater/capella-updates-94/update_5601823_5609044.json"
    let pathToNewUpdate = "vendor/eth2-light-client-updates/prater/capella-updates-94/update_5609044_5609069.json"
    let domain = "0x07000000628941ef21d1fe8c7134720add10bb91e3b02c007e0046d2472c6695"

  test "check verifyProof for bncurve":
    assert verifyProof(pathToKey, pathToProof, pathToLastUpdate,
                       pathToNewUpdate, domain)
#Will me added with another PR
  # test "check verifyProof for constantine":
  #   assert verifyProofConstantine(pathToKey, pathToProof, pathToLastUpdate,
  #                                 pathToNewUpdate, domain)


