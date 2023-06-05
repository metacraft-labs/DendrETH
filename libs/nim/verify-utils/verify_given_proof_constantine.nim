import
  nimcrypto/[sha2, hash, utils],
  stew/byteutils,
  std/[strutils,json]

import # constantine imports
  constantine/math/pairings/pairings_bn,
  tests/math/t_pairing_template,
  constantine/math/io/[io_ec, io_fields, io_bigints],
  constantine/math/elliptic/ec_scalar_mul,
  constantine/math/config/type_bigint

type
  IC* = array[5, ECP_ShortW_Aff[Fp[BN254_Snarks], G1]]

  VerificationKey* = object
    alpha*: ECP_ShortW_Aff[Fp[BN254_Snarks], G1]
    beta*, gamma*, delta*: ECP_ShortW_Aff[Fp2[BN254_Snarks], G2]
    ic*: IC

  Proof* = object
    a*, c*: ECP_ShortW_Aff[Fp[BN254_Snarks], G1]
    b*: ECP_ShortW_Aff[Fp2[BN254_Snarks], G2]

  Header* = object
    head*: Fr[BN254_Snarks]
    tail*: Fr[BN254_Snarks]

  Input* = object
    data*: ECP_ShortW_Aff[Fp[BN254_Snarks], G1]

proc concatArrays*(
    currentHeaderHash: array[32, byte], newOptimisticHeader: array[32, byte],
    newFinalizedHeader: array[32, byte], newExecutionStateRoot: array[32, byte],
    zerosSlotBuffer: array[24, byte], currentSlot: array[8, byte],
    domain: array[32, byte]): array[192, byte] =
  var res: array[192, byte]
  res[0..31] = currentHeaderHash
  res[32..63] = newOptimisticHeader
  res[64..95] = newFinalizedHeader
  res[96..127] = newExecutionStateRoot
  res[128..151] = zerosSlotBuffer
  res[152..159] = currentSlot
  res[160..191] = domain

  res

proc hashHeaders*(
    currentHeaderHash: array[32, byte], newOptimisticHeader: array[32, byte],
    newFinalizedHeader: array[32, byte], newExecutionStateRoot: array[32, byte],
    zerosSlotBuffer: array[24, byte], currentSlot: array[8, byte],
    domain: array[32, byte]): array[32, byte]  =
  let concat = (concatArrays(currentHeaderHash, newOptimisticHeader,
                             newFinalizedHeader, newExecutionStateRoot,
                             zerosSlotBuffer, currentSlot, domain))


  let hash = sha2.sha256.digest(concat)
  hash.data

proc TwoOnPower*(power: int): int =
  var output = 1
  for i in 1..power:
    output *= 2
  output

proc decToBitArray*(number: int): array[8, int] =
  var copyNum = number
  var bitmask: array[8, int]
  for i in countdown(7,0):
    bitmask[7-i] = copyNum div TwoOnPower(i)
    copyNum = (copyNum mod TwoOnPower(i))
  bitmask

proc bitArrayToByte*(arr: array[8, int]): byte =
  var outNum = 0
  for i in 0..7:
    outNum += TwoOnPower(i)*arr[7-i]
  outNum.byte

proc headerFromSeq*(bigNum: seq): Header =
  var firstNumInBits: array[256, int]
  for i in 0..2:
    firstNumInBits[i] = 0

  var secondNumInBits: array[256, int]
  for i in 0..252:
    secondNumInBits[i] = 0

  for i in 0..30:
    var tempBitArray = decToBitArray(bigNum[i].int)
    for j in 0..7:
      firstNumInBits[i*8+j+3] = tempBitArray[j]

  var tempBitArray = decToBitArray(bigNum[31].int)
  for i in 0..4:
    firstNumInBits[251+i] = tempBitArray[i]
  for i in 5..7:
    secondNumInBits[248+i] = tempBitArray[i]

  var firstNumInBytes: array[32, byte]
  for i in 0..31:
    firstNumInBytes[i] = bitArrayToByte([firstNumInBits[i*8],firstNumInBits[i*8+1],firstNumInBits[i*8+2],firstNumInBits[i*8+3],firstNumInBits[i*8+4],firstNumInBits[i*8+5],firstNumInBits[i*8+6],firstNumInBits[i*8+7]])

  var secondNumInBytes: array[32, byte]
  for i in 0..30:
    secondNumInBytes[i] = 0.byte
  secondNumInBytes[31] = bitArrayToByte([secondNumInBits[248],secondNumInBits[249],secondNumInBits[250],secondNumInBits[251],secondNumInBits[252],secondNumInBits[253],secondNumInBits[254],secondNumInBits[255]])

  var
    head: Fr[BN254_Snarks]
    tail: Fr[BN254_Snarks]
  head.fromHex(toHex(firstNumInBytes))
  tail.fromHex(toHex(secondNumInBytes))

  Header(head: head, tail: tail)

proc createVerificationKey*(path: string): VerificationKey =
  let vk = parseFile(path)

  let vkAlpha1 = ECP_ShortW_Aff[Fp[BN254_Snarks], G1].fromHex(BigInt[255].fromDecimal(vk["vk_alpha_1"][0].str).toHex(),BigInt[255].fromDecimal(vk["vk_alpha_1"][1].str).toHex())
  let vkBeta2 = ECP_ShortW_Aff[Fp2[BN254_Snarks], G2].fromHex(BigInt[255].fromDecimal(vk["vk_beta_2"][0][0].str).toHex(),BigInt[255].fromDecimal(vk["vk_beta_2"][0][1].str).toHex(),BigInt[255].fromDecimal(vk["vk_beta_2"][1][0].str).toHex(),BigInt[255].fromDecimal(vk["vk_beta_2"][1][1].str).toHex())
  let vkGamma2 = ECP_ShortW_Aff[Fp2[BN254_Snarks], G2].fromHex(BigInt[255].fromDecimal(vk["vk_gamma_2"][0][0].str).toHex(),BigInt[255].fromDecimal(vk["vk_gamma_2"][0][1].str).toHex(),BigInt[255].fromDecimal(vk["vk_gamma_2"][1][0].str).toHex(),BigInt[255].fromDecimal(vk["vk_gamma_2"][1][1].str).toHex())
  let vkDelta2 = ECP_ShortW_Aff[Fp2[BN254_Snarks], G2].fromHex(BigInt[255].fromDecimal(vk["vk_delta_2"][0][0].str).toHex(),BigInt[255].fromDecimal(vk["vk_delta_2"][0][1].str).toHex(),BigInt[255].fromDecimal(vk["vk_delta_2"][1][0].str).toHex(),BigInt[255].fromDecimal(vk["vk_delta_2"][1][1].str).toHex())

  var icArr: IC
  var counter = 0
  for el in vk["IC"]:
    let ic = ECP_ShortW_Aff[Fp[BN254_Snarks], G1].fromHex(BigInt[255].fromDecimal(el[0].str).toHex(),BigInt[255].fromDecimal(el[1].str).toHex())

    icArr[counter] = ic
    counter+=1

  VerificationKey(alpha:vkAlpha1, beta:vkBeta2, gamma:vkGamma2, delta:vkDelta2, ic:icArr)

proc createProof*(path: string): Proof =
  let proof = parseFile(path)

  let a = ECP_ShortW_Aff[Fp[BN254_Snarks], G1].fromHex(BigInt[255].fromDecimal(proof["pi_a"][0].str).toHex(),BigInt[255].fromDecimal(proof["pi_a"][1].str).toHex())
  let b = ECP_ShortW_Aff[Fp2[BN254_Snarks], G2].fromHex(BigInt[255].fromDecimal(proof["pi_b"][0][0].str).toHex(),BigInt[255].fromDecimal(proof["pi_b"][0][1].str).toHex(),
  BigInt[255].fromDecimal(proof["pi_b"][1][0].str).toHex(),BigInt[255].fromDecimal(proof["pi_b"][1][1].str).toHex())
  let c = ECP_ShortW_Aff[Fp[BN254_Snarks], G1].fromHex(BigInt[255].fromDecimal(proof["pi_c"][0].str).toHex(),BigInt[255].fromDecimal(proof["pi_c"][1].str).toHex())

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

  var preparedInputs:Input
  var ic0Prj: ECP_ShortW_Prj[Fp[BN254_Snarks], G1]
  var ic1Prj: ECP_ShortW_Prj[Fp[BN254_Snarks], G1]
  var ic2Prj: ECP_ShortW_Prj[Fp[BN254_Snarks], G1]
  fromAffine(ic0Prj, vk.ic[0])
  fromAffine(ic1Prj, vk.ic[1])
  fromAffine(ic2Prj, vk.ic[2])
  scalarMul(ic1Prj,toBig(header.head))
  scalarMul(ic2Prj,toBig(header.tail))
  ic0Prj += ic1Prj
  ic0Prj += ic2Prj
  affine(preparedInputs.data,ic0Prj)

  var aBPairing: Fp12[BN254_Snarks]
  pairing_bn[BN254_Snarks](aBPairing,prf.a,prf.b)
  var alphaBetaPairing: Fp12[BN254_Snarks]
  pairing_bn[BN254_Snarks](alphaBetaPairing,vk.alpha,vk.beta)
  var preparedInputsGammaPairing: Fp12[BN254_Snarks]
  pairing_bn[BN254_Snarks](preparedInputsGammaPairing,preparedInputs.data,vk.gamma)
  var proofCVkDeltaPairing: Fp12[BN254_Snarks]
  pairing_bn[BN254_Snarks](proofCVkDeltaPairing,prf.c,vk.delta)

  var sum:Fp12[BN254_Snarks]
  prod(sum, alphaBetaPairing, preparedInputsGammaPairing)
  prod(sum, sum, proofCVkDeltaPairing)

  (sum == aBPairing).bool

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
