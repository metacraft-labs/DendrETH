import
  stew/byteutils,
  std/[strutils,json]

import
  ../../../contracts/cosmos/verifier/lib/nim/verify/verify_helpers

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

  let fq0 = FQ.fromString("0")
  let fq1 = FQ.fromString("1")

  let alpha0 = FQ.fromString(vk["vk_alpha_1"][0].str)
  let alpha1 = FQ.fromString(vk["vk_alpha_1"][1].str)
  let vkAlpha1 = Point[G1](x: alpha0, y: alpha1, z: fq1)

  let beta00 = FQ.fromString(vk["vk_beta_2"][0][0].str)
  let beta01 = FQ.fromString(vk["vk_beta_2"][0][1].str)
  let beta10 = FQ.fromString(vk["vk_beta_2"][1][0].str)
  let beta11 = FQ.fromString(vk["vk_beta_2"][1][1].str)
  let beta0 = FQ2(c0: beta00, c1: beta01)
  let beta1 = FQ2(c0: beta10, c1: beta11)
  let vkBeta2 = Point[G2](x: beta0, y: beta1, z: FQ2(c0: fq1, c1: fq0))

  let gamma00 = FQ.fromString(vk["vk_gamma_2"][0][0].str)
  let gamma01 = FQ.fromString(vk["vk_gamma_2"][0][1].str)
  let gamma10 = FQ.fromString(vk["vk_gamma_2"][1][0].str)
  let gamma11 = FQ.fromString(vk["vk_gamma_2"][1][1].str)
  let gamma0 = FQ2(c0: gamma00, c1: gamma01)
  let gamma1 = FQ2(c0: gamma10, c1: gamma11)
  let vkGamma2 = Point[G2](x: gamma0, y: gamma1, z: FQ2(c0: fq1, c1: fq0))

  let delta00 = FQ.fromString(vk["vk_delta_2"][0][0].str)
  let delta01 = FQ.fromString(vk["vk_delta_2"][0][1].str)
  let delta10 = FQ.fromString(vk["vk_delta_2"][1][0].str)
  let delta11 = FQ.fromString(vk["vk_delta_2"][1][1].str)
  let delta0 = FQ2(c0: delta00, c1: delta01)
  let delta1 = FQ2(c0: delta10, c1: delta11)
  let vkDelta2 = Point[G2](x: delta0, y: delta1, z: FQ2(c0: fq1, c1: fq0))

  var icArr: IC
  var counter = 0
  for el in vk["IC"]:
    let ic0 = FQ.fromString(el[0].str)
    let ic1 = FQ.fromString(el[1].str)
    let ic = Point[G1](x: ic0, y: ic1, z: fq1)
    icArr[counter] = ic
    counter+=1

  VerificationKey(alpha:vkAlpha1, beta:vkBeta2, gamma:vkGamma2, delta:vkDelta2, ic:icArr)

proc createProof*(path: string): Proof =
  let proof = parseFile(path)

  let fq0 = FQ.fromString("0")
  let fq1 = FQ.fromString("1")

  let a0 = FQ.fromString(proof["pi_a"][0].str)
  let a1 = FQ.fromString(proof["pi_a"][1].str)
  let a = Point[G1](x: a0, y: a1, z: fq1)

  let b00 = FQ.fromString(proof["pi_b"][0][0].str)
  let b01 = FQ.fromString(proof["pi_b"][0][1].str)
  let b10 = FQ.fromString(proof["pi_b"][1][0].str)
  let b11 = FQ.fromString(proof["pi_b"][1][1].str)
  let b0 = FQ2(c0: b00, c1: b01)
  let b1 = FQ2(c0: b10, c1: b11)
  let b = Point[G2](x: b0, y: b1, z: FQ2(c0: fq1, c1: fq0))

  let c0 = FQ.fromString(proof["pi_c"][0].str)
  let c1 = FQ.fromString(proof["pi_c"][1].str)
  let c = Point[G1](x: c0, y: c1, z: fq1)

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
  let sha256ofHashes = hashHeaders(currentHeaderHash,
                                   newOptimisticHeader,
                                   newFinalizedHeader,
                                   newExecutionStateRoot,
                                   zerosSlotBuffer,
                                   currentSlot,
                                   domain)

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

proc verifyProof*(pathToKey:string,
                  pathToProof:string,
                  pathToLastUpdate:string,
                  pathToNewUpdate:string,
                  domain:string): bool =
  let vkey = createVerificationKey(pathToKey)
  let proof = createProof(pathToProof)
  let header = createHeader(pathToLastUpdate,pathToNewUpdate,domain)

  makePairsAndVerify(vkey,proof,header)
