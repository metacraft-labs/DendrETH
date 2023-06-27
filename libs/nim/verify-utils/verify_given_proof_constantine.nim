import
  stew/byteutils,
  std/[strutils,json]

import # constantine imports
  constantine/math/pairings/pairings_bn,
  tests/math/t_pairing_template,
  constantine/math/io/[io_ec, io_fields, io_bigints],
  constantine/math/elliptic/ec_scalar_mul,
  constantine/math/config/type_bigint

import
  ../../../contracts/cosmos/verifier-constantine/lib/nim/verify/verify_helpers

type
  IC* = array[3, ECP_ShortW_Aff[Fp[BN254_Snarks], G1]]

  VerificationKey* = object
    alpha*: ECP_ShortW_Aff[Fp[BN254_Snarks], G1]
    beta*, gamma*, delta*: ECP_ShortW_Aff[Fp2[BN254_Snarks], G2]
    ic*: IC

  Proof* = object
    a*, c*: ECP_ShortW_Aff[Fp[BN254_Snarks], G1]
    b*: ECP_ShortW_Aff[Fp2[BN254_Snarks], G2]

  Input* = object
    data*: ECP_ShortW_Aff[Fp[BN254_Snarks], G1]

proc createVerificationKey*(path: string): VerificationKey =
  let vk = parseFile(path)

  let alpha0 = BigInt[255].fromDecimal(vk["vk_alpha_1"][0].str).toHex()
  let alpha1 = BigInt[255].fromDecimal(vk["vk_alpha_1"][1].str).toHex()
  let vkAlpha1 = ECP_ShortW_Aff[Fp[BN254_Snarks], G1].fromHex(alpha0, alpha1)

  let beta00 = BigInt[255].fromDecimal(vk["vk_beta_2"][0][0].str).toHex()
  let beta01 = BigInt[255].fromDecimal(vk["vk_beta_2"][0][1].str).toHex()
  let beta10 = BigInt[255].fromDecimal(vk["vk_beta_2"][1][0].str).toHex()
  let beta11 = BigInt[255].fromDecimal(vk["vk_beta_2"][1][1].str).toHex()
  let vkBeta2 = ECP_ShortW_Aff[Fp2[BN254_Snarks], G2].fromHex(beta00, beta01, beta10, beta11)

  let gamma00 = BigInt[255].fromDecimal(vk["vk_gamma_2"][0][0].str).toHex()
  let gamma01 = BigInt[255].fromDecimal(vk["vk_gamma_2"][0][1].str).toHex()
  let gamma10 = BigInt[255].fromDecimal(vk["vk_gamma_2"][1][0].str).toHex()
  let gamma11 = BigInt[255].fromDecimal(vk["vk_gamma_2"][1][1].str).toHex()
  let vkGamma2 = ECP_ShortW_Aff[Fp2[BN254_Snarks], G2].fromHex(gamma00, gamma01, gamma10, gamma11)

  let delta00 = BigInt[255].fromDecimal(vk["vk_delta_2"][0][0].str).toHex()
  let delta01 = BigInt[255].fromDecimal(vk["vk_delta_2"][0][1].str).toHex()
  let delta10 = BigInt[255].fromDecimal(vk["vk_delta_2"][1][0].str).toHex()
  let delta11 = BigInt[255].fromDecimal(vk["vk_delta_2"][1][1].str).toHex()
  let vkDelta2 = ECP_ShortW_Aff[Fp2[BN254_Snarks], G2].fromHex(delta00, delta01, delta10, delta11)

  var icArr: IC
  var counter = 0
  for el in vk["IC"]:
    let ic0 = BigInt[255].fromDecimal(el[0].str).toHex()
    let ic1 = BigInt[255].fromDecimal(el[1].str).toHex()
    let ic = ECP_ShortW_Aff[Fp[BN254_Snarks], G1].fromHex(ic0, ic1)

    icArr[counter] = ic
    counter+=1

  VerificationKey(alpha:vkAlpha1, beta:vkBeta2, gamma:vkGamma2, delta:vkDelta2, ic:icArr)

proc createProof*(path: string): Proof =
  let proof = parseFile(path)

  let a0 = BigInt[255].fromDecimal(proof["pi_a"][0].str).toHex()
  let a1 = BigInt[255].fromDecimal(proof["pi_a"][1].str).toHex()
  let a = ECP_ShortW_Aff[Fp[BN254_Snarks], G1].fromHex(a0, a1)

  let b00 = BigInt[255].fromDecimal(proof["pi_b"][0][0].str).toHex()
  let b01 = BigInt[255].fromDecimal(proof["pi_b"][0][1].str).toHex()
  let b10 = BigInt[255].fromDecimal(proof["pi_b"][1][0].str).toHex()
  let b11 = BigInt[255].fromDecimal(proof["pi_b"][1][1].str).toHex()
  let b = ECP_ShortW_Aff[Fp2[BN254_Snarks], G2].fromHex(b00, b01, b10, b11)

  let c0 = BigInt[255].fromDecimal(proof["pi_c"][0].str).toHex()
  let c1 = BigInt[255].fromDecimal(proof["pi_c"][1].str).toHex()
  let c = ECP_ShortW_Aff[Fp[BN254_Snarks], G1].fromHex(c0, c1)

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
  var preparedInputs:Input
  var ic0Prj: ECP_ShortW_Prj[Fp[BN254_Snarks], G1]
  var ic1Prj: ECP_ShortW_Prj[Fp[BN254_Snarks], G1]
  var ic2Prj: ECP_ShortW_Prj[Fp[BN254_Snarks], G1]
  fromAffine(ic0Prj, vk.ic[0])
  fromAffine(ic1Prj, vk.ic[1])
  fromAffine(ic2Prj, vk.ic[2])
  scalarMul(ic1Prj, toBig(header.head))
  scalarMul(ic2Prj, toBig(header.tail))
  ic0Prj += ic1Prj
  ic0Prj += ic2Prj
  affine(preparedInputs.data, ic0Prj)

  var aBPairing: Fp12[BN254_Snarks]
  pairing_bn[BN254_Snarks](aBPairing, prf.a, prf.b)
  var alphaBetaPairing: Fp12[BN254_Snarks]
  pairing_bn[BN254_Snarks](alphaBetaPairing, vk.alpha, vk.beta)
  var preparedInputsGammaPairing: Fp12[BN254_Snarks]
  pairing_bn[BN254_Snarks](preparedInputsGammaPairing, preparedInputs.data, vk.gamma)
  var proofCVkDeltaPairing: Fp12[BN254_Snarks]
  pairing_bn[BN254_Snarks](proofCVkDeltaPairing, prf.c, vk.delta)

  var sum:Fp12[BN254_Snarks]
  prod(sum, alphaBetaPairing, preparedInputsGammaPairing)
  prod(sum, sum, proofCVkDeltaPairing)

  (sum == aBPairing).bool

proc verifyProofConstantine*(pathToKey:string,
                             pathToProof:string,
                             pathToLastUpdate:string,
                             pathToNewUpdate:string,
                             domain:string): bool =
  let vkey = createVerificationKey(pathToKey)
  let proof = createProof(pathToProof)
  let header = createHeader(pathToLastUpdate,pathToNewUpdate,domain)

  makePairsAndVerify(vkey, proof, header)
