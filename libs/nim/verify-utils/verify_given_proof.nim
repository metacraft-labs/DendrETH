import
  ../../../contracts/cosmos/verifier/lib/nim/verify/verify_helpers,
  stew/byteutils,
  std/[strutils,json]

type
  IC* = array[3, Point[G1]]

  VerificationKey* = object
    alpha*: Point[G1]
    beta*, gamma*, delta*: Point[G2]
    ic*: IC

  Proof* = object
    a*, c*: Point[G1]
    b*: Point[G2]

  Input* = object
    data*: Point[G1]

proc createVerificationKey*(path: string): VerificationKey =
  let vk = parseFile(path)

  let vkAlpha1 = Point[G1](x: FQ.fromString(vk["vk_alpha_1"][0].str), y: FQ.fromString(vk["vk_alpha_1"][1].str), z: FQ.fromString("1"))
  let vkBeta2 = Point[G2](x: FQ2(c0: FQ.fromString(vk["vk_beta_2"][0][0].str),  c1: FQ.fromString(vk["vk_beta_2"][0][1].str)), y: FQ2(c0: FQ.fromString(vk["vk_beta_2"][1][0].str), c1: FQ.fromString(vk["vk_beta_2"][1][1].str)), z: FQ2(c0: FQ.fromString("1"), c1: FQ.fromString("0")))
  let vkGamma2 = Point[G2](x: FQ2(c0: FQ.fromString(vk["vk_gamma_2"][0][0].str),  c1: FQ.fromString(vk["vk_gamma_2"][0][1].str)), y: FQ2(c0: FQ.fromString(vk["vk_gamma_2"][1][0].str), c1: FQ.fromString(vk["vk_gamma_2"][1][1].str)), z: FQ2(c0: FQ.fromString("1"), c1: FQ.fromString("0")))
  let vkDelta2 = Point[G2](x: FQ2(c0: FQ.fromString(vk["vk_delta_2"][0][0].str),  c1: FQ.fromString(vk["vk_delta_2"][0][1].str)), y: FQ2(c0: FQ.fromString(vk["vk_delta_2"][1][0].str), c1: FQ.fromString(vk["vk_delta_2"][1][1].str)), z: FQ2(c0: FQ.fromString("1"), c1: FQ.fromString("0")))

  var icArr: IC
  var counter = 0
  for el in vk["IC"]:
    let ic = Point[G1](x: FQ.fromString(el[0].str), y: FQ.fromString(el[1].str), z: FQ.fromString("1"))
    icArr[counter] = ic
    counter+=1

  VerificationKey(alpha:vkAlpha1, beta:vkBeta2, gamma:vkGamma2, delta:vkDelta2, ic:icArr)

proc createProof*(path: string): Proof =
  let proof = parseFile(path)

  let a = Point[G1](x: FQ.fromString(proof["pi_a"][0].str), y: FQ.fromString(proof["pi_a"][1].str), z: FQ.fromString("1"))
  let b = Point[G2](x: FQ2(c0: FQ.fromString(proof["pi_b"][0][0].str),  c1: FQ.fromString(proof["pi_b"][0][1].str)), y: FQ2(c0: FQ.fromString(proof["pi_b"][1][0].str), c1: FQ.fromString(proof["pi_b"][1][1].str)), z: FQ2(c0: FQ.fromString("1"), c1: FQ.fromString("0")))
  let c = Point[G1](x: FQ.fromString(proof["pi_c"][0].str), y: FQ.fromString(proof["pi_c"][1].str), z: FQ.fromString("1"))

  Proof(a:a, b:b, c:c)

proc createHeader*(pathCurrentHeader: string, updatePath: string, domain: string): Header =
  let currentHeaderHashJSON = parseFile(pathCurrentHeader)
  let updateJson = parseFile(updatePath)
  let currentHeaderHash = hexToByteArray[32](currentHeaderHashJSON["attestedHeaderRoot"].str)
  let newOptimisticHeader = hexToByteArray[32](updateJson["attestedHeaderRoot"].str)
  let newFinalizedHeader = hexToByteArray[32](updateJson["finalizedHeaderRoot"].str)
  let newExecutionStateRoot = hexToByteArray[32](updateJson["finalizedExecutionStateRoot"].str)
  var slot = updateJson["attestedHeaderSlot"].getInt().toHex()
  var currentSlot = hexToByteArray[8](slot)
  var domain = hexToByteArray[32](domain)

  var zerosSlotBuffer: array[24, byte]
  for i in 0..23:
    zerosSlotBuffer[i] = 0
  let sha256ofHashes = hashHeaders(currentHeaderHash, newOptimisticHeader, newFinalizedHeader, newExecutionStateRoot, zerosSlotBuffer, currentSlot, domain)

  headerFromSeq(@sha256ofHashes)

proc makePairsAndVerify*(vk: VerificationKey,
                         prf: Proof,
                         header: Header): bool =

  var preparedInputs = Input(data:vk.ic[0])
  preparedInputs.data = preparedInputs.data + (vk.ic[1] * header.head)
  preparedInputs.data = preparedInputs.data + (vk.ic[2] * header.tail)

  let aBPairing = pairing(prf.a, prf.b)
  let alphaBetaPairingP = pairing(vk.alpha, vk.beta)
  let preparedInputsGammaPairing = pairing(preparedInputs.data, vk.gamma)
  let proofCVkDeltaPairing = pairing(prf.c, vk.delta)
  let sum = alphaBetaPairingP * preparedInputsGammaPairing * proofCVkDeltaPairing;

  aBPairing == sum

proc VerifyProofByPaths*(pathToKey:string, pathToProof:string, pathToLastUpdate:string, pathToNewUpdate:string, domain:string): bool =
  let vkey = createVerificationKey(pathToKey)
  let proof = createProof(pathToProof)
  let header = createHeader(pathToLastUpdate,pathToNewUpdate,domain)

  makePairsAndVerify(vkey,proof,header)

# Example data and usage

# let pathToKey = "vendor/eth2-light-client-updates/prater/capella-updates-94/vk.json"
# let pathToProof = "vendor/eth2-light-client-updates/prater/capella-updates-94/proof_5609044_5609069.json"
# let domain = "0x07000000628941ef21d1fe8c7134720add10bb91e3b02c007e0046d2472c6695"
# let pathToLastUpdate = "vendor/eth2-light-client-updates/prater/capella-updates-94/update_5601823_5609044.json"
# let pathToNewUpdate = "vendor/eth2-light-client-updates/prater/capella-updates-94/update_5609044_5609069.json"

# let vkey = createVerificationKey(pathToKey)
# let proof = createProof(pathToProof)
# let header = createHeader(pathToLastUpdate,pathToNewUpdate,domain)

# if makePairsAndVerify(vkey,proof,header):
#   echo "Correct update!"
# else:
#   echo "Incorrect update!"
