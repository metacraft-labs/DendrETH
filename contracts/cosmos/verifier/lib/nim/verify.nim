when defined(emcc):
  {.pragma: wasmPragma, cdecl, exportc, dynlib, codegenDecl: "__attribute__((used)) $# $#$#".}
else:
  {.pragma: wasmPragma, cdecl, exportc, dynlib.}
import
  bncurve/group_operations,
  ../../../../../vendor/nimcrypto/nimcrypto/[sha2, hash, utils],
  stew/byteutils,
  sequtils,std/json,std/marshal

type
  IC* = array[5, Point[G1]]

  VerificationKey* = object
    alpha*: Point[G1]
    beta*, gamma*, delta*: Point[G2]
    ic*: IC

  Proof* = object
    a*, c*: Point[G1]
    b*: Point[G2]

  Header* = object
    head*: FR
    tail*: FR

  Input* = object
    data*: Point[G1]


proc seqToArray(seqHash: seq[byte]): array[32, byte] =
  var res: array[32, byte]
  for index, el in seqHash:
    res[index] = seqHash[index]
  res

proc concatMy(currentHeaderHash: array[32, byte], newOptimisticHeader: array[32, byte], newFinalizedHeader: array[32, byte], newExecutionStateRoot: array[32, byte]): array[128, byte] =
  var res: array[128, byte]
  res[0..31] = currentHeaderHash
  res[32..63] = newOptimisticHeader
  res[64..95] = newFinalizedHeader
  res[96..127] = newExecutionStateRoot
  res

proc hashHeaders*(currentHeaderHash: array[32, byte], newOptimisticHeader: array[32, byte], newFinalizedHeader: array[32, byte], newExecutionStateRoot: array[32, byte]): array[32, byte]  =
  let concat = (concatMy(currentHeaderHash,
                         newOptimisticHeader,
                         newFinalizedHeader,
                         newExecutionStateRoot))


  let hash = sha2.sha256.digest(concat)
  hash.data


proc TwoOnPower*(power: int): int =
  var output = 1
  for i in 1..power:
    output *= 2
  output

proc decToBitArray(number: int): array[8, int] =
  var copyNum = number
  var bitmask: array[8, int]
  for i in countdown(7,0):
    bitmask[7-i] = copyNum div TwoOnPower(i)
    copyNum = (copyNum mod TwoOnPower(i))
  bitmask

proc bitArrayToByte(arr: array[8, int]): byte =
  var outNum = 0
  for i in 0..7:
    outNum += TwoOnPower(i)*arr[7-i]
  outNum.byte

proc headerFromSeq(bigNum: seq): Header {.wasmPragma.} =
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
    headBNU: BNU256
    tailBNU: BNU256
  discard headBNU.fromBytes(firstNumInBytes)
  discard tailBNU.fromBytes(secondNumInBytes)
  discard headBNU.fromHexString(toString(headBNU))
  discard tailBNU.fromHexString(toString(tailBNU))
  var headFR: typedesc[FR]
  var tailFR: typedesc[FR]
  var head = init(headFR,headBNU)
  var tail = init(tailFR,tailBNU)

  Header(head: head.get, tail: tail.get)

proc makePairsAndVerify*(vk: ref VerificationKey,
                         prf: ref Proof,
                         currentHeaderHash: var array[32, byte],
                         newOptimisticHeader: array[32, byte],
                         newFinalizedHeader: array[32, byte],
                         newExecutionStateRoot: array[32, byte]): bool {.wasmPragma.} =

  let hasher = hashHeaders(currentHeaderHash, newOptimisticHeader, newFinalizedHeader, newExecutionStateRoot)
  let header = headerFromSeq(@hasher)

  var preparedInputs = Input(data:vk[].ic[0])
  preparedInputs.data = preparedInputs.data + (vk[].ic[1] * header.head)
  preparedInputs.data = preparedInputs.data + (vk[].ic[2] * header.tail)

  let aBPairing = pairing(prf[].a, prf[].b)
  let alphaBetaPairingP = pairing(vk[].alpha, vk[].beta)
  let preparedInputsGammaPairing = pairing(preparedInputs.data, vk[].gamma)
  let proofCVkDeltaPairing = pairing(prf[].c, vk[].delta)
  let sum = alphaBetaPairingP * preparedInputsGammaPairing * proofCVkDeltaPairing;


  if aBPairing == sum:
    currentHeaderHash = newOptimisticHeader
    return true
  else:
    return false
  aBPairing == sum

# proc makePairsAndVerifyTest*(vk: VerificationKey,
#                          prf: Proof,
#                          currentHeaderHash: var array[32, byte],
#                          newOptimisticHeader: array[32, byte],
#                          newFinalizedHeader: array[32, byte],
#                          newExecutionStateRoot: array[32, byte]): bool {.wasmPragma.} =

#   # var hasher: array[32, byte]
#   let hasher = hashHeaders(currentHeaderHash, newOptimisticHeader, newFinalizedHeader, newExecutionStateRoot)
#   let header = headerFromSeq(@hasher)

#   var preparedInputs = Input(data:vk.ic[0])
#   preparedInputs.data = preparedInputs.data + (vk.ic[1] * header.head)
#   preparedInputs.data = preparedInputs.data + (vk.ic[2] * header.tail)

#   let aBPairing = pairing(prf.a, prf.b)
#   let alphaBetaPairingP = pairing(vk.alpha, vk.beta)
#   let preparedInputsGammaPairing = pairing(preparedInputs.data, vk.gamma)
#   let proofCVkDeltaPairing = pairing(prf.c, vk.delta)
#   let sum = alphaBetaPairingP * preparedInputsGammaPairing * proofCVkDeltaPairing;
#   # echo typeof sum
#   # # var a: array[32, byte] = toArray(32, seqHash)
#   # # let res = seqToArray(seqHash)
#   # # currentHeaderHash = seqHash
#   # # return false
#   # if aBPairing == sum:
#   #   currentHeaderHash = seqToArray(seqHash)
#   #   return true
#   # else:
#   #   false
#   # # aBPairing == sum
#   # echo seqHash
#   # seqToArray(seqHash)
#   aBPairing == sum


# proc createVerificationKey*(): VerificationKey =
#   let vk = parseFile("vkey.json")

#   let vkAlpha1 = Point[G1](x: FQ.fromString(vk["vk_alpha_1"][0].str), y: FQ.fromString(vk["vk_alpha_1"][1].str), z: FQ.fromString("1"))
#   let vkBeta2 = Point[G2](x: FQ2(c0: FQ.fromString(vk["vk_beta_2"][0][0].str),  c1: FQ.fromString(vk["vk_beta_2"][0][1].str)), y: FQ2(c0: FQ.fromString(vk["vk_beta_2"][1][0].str), c1: FQ.fromString(vk["vk_beta_2"][1][1].str)), z: FQ2(c0: FQ.fromString("1"), c1: FQ.fromString("0")))
#   let vkGamma2 = Point[G2](x: FQ2(c0: FQ.fromString(vk["vk_gamma_2"][0][0].str),  c1: FQ.fromString(vk["vk_gamma_2"][0][1].str)), y: FQ2(c0: FQ.fromString(vk["vk_gamma_2"][1][0].str), c1: FQ.fromString(vk["vk_gamma_2"][1][1].str)), z: FQ2(c0: FQ.fromString("1"), c1: FQ.fromString("0")))
#   let vkDelta2 = Point[G2](x: FQ2(c0: FQ.fromString(vk["vk_delta_2"][0][0].str),  c1: FQ.fromString(vk["vk_delta_2"][0][1].str)), y: FQ2(c0: FQ.fromString(vk["vk_delta_2"][1][0].str), c1: FQ.fromString(vk["vk_delta_2"][1][1].str)), z: FQ2(c0: FQ.fromString("1"), c1: FQ.fromString("0")))

#   var icArr: IC
#   var counter = 0
#   for el in vk["IC"]:
#     let ic = Point[G1](x: FQ.fromString(el[0].str), y: FQ.fromString(el[1].str), z: FQ.fromString("1"))
#     icArr[counter] = ic
#     counter+=1

#   VerificationKey(alpha:vkAlpha1, beta:vkBeta2, gamma:vkGamma2, delta:vkDelta2, ic:icArr)


# proc createProof*(): Proof =
#   let proof = parseFile("proof2.json")

#   let a = Point[G1](x: FQ.fromString(proof["pi_a"][0].str), y: FQ.fromString(proof["pi_a"][1].str), z: FQ.fromString("1"))
#   let b = Point[G2](x: FQ2(c0: FQ.fromString(proof["pi_b"][0][0].str),  c1: FQ.fromString(proof["pi_b"][0][1].str)), y: FQ2(c0: FQ.fromString(proof["pi_b"][1][0].str), c1: FQ.fromString(proof["pi_b"][1][1].str)), z: FQ2(c0: FQ.fromString("1"), c1: FQ.fromString("0")))
#   let c = Point[G1](x: FQ.fromString(proof["pi_c"][0].str), y: FQ.fromString(proof["pi_c"][1].str), z: FQ.fromString("1"))

#   Proof(a:a, b:b, c:c)

# # let header = Header(head: Fr.fromString("4806037384478563096050653759846648483652627660569280784719614036894771231500"),tail:Fr.fromString("3"))

# # var hex = hexToByteArray[32]("0xc43d94aaea1342f8e551d9a5e6fe95b7ebb013142acf1e2628ad381e5c713316")
# # let hex2 = hexToByteArray[32]("0x51e177b2e6e99ae2b2179f54a471031622713a321d407b56fc3293c0d3d634bb")
# # let hex3 = hexToByteArray[32]("0x320129973260d56499e4a85e436ca57775be7b024ad04f7aee97019628d2b1cb")
# # let hex4 =  hexToByteArray[32]("0x79a462ed5b52be97b8c887f92c2111b2a4d04cc3fef85ce1e5fcd9bf2e958f7b")
# var hex = [(byte)196, 61, 148, 170, 234, 19, 66, 248, 229, 81, 217, 165, 230, 254, 149, 183, 235, 176, 19, 20, 42, 207, 30, 38, 40, 173, 56, 30, 92, 113, 51, 22]
# var hex2 = [(byte)81, 225, 119, 178, 230, 233, 154, 226, 178, 23, 159, 84, 164, 113, 3, 22, 34, 113, 58, 50, 29, 64, 123, 86, 252, 50, 147, 192, 211, 214, 52, 187]
# let hex3 = [(byte)50, 1, 41, 151, 50, 96, 213, 100, 153, 228, 168, 94, 67, 108, 165, 119, 117, 190, 123, 2, 74, 208, 79, 122, 238, 151, 1, 150, 40, 210, 177, 203]
# let hex4 = [(byte)121, 164, 98, 237, 91, 82, 190, 151, 184, 200, 135, 249, 44, 33, 17, 178, 164, 208, 76, 195, 254, 248, 92, 225, 229, 252, 217, 191, 46, 149, 143, 123]

# var hex5 = [(byte)80, 230, 197, 221, 210, 33, 167, 93, 47, 37, 132, 33, 28, 24, 15, 239, 0, 176, 64, 243, 45, 5, 253, 195, 102, 152, 51, 70, 244, 82, 244, 13]
# let hex6 = hexToByteArray[32]("0x50e0ee956f582c81f123a2517872c8238504e98ca88a6834b21d2d58a26267db")
# let hex7 = hexToByteArray[32]("0x47b54c2939ed8025497d355ca8046b2535e521fed30467b6c62aa1509522e39c")
# let hex8 = hexToByteArray[32]("0x3c651714093ac5bbae9d25249a89e6a56887d0f9f5c76594b796553e491f64ca")

# # var sb: array[32, byte]
# echo makePairsAndVerifyTest(createVerificationKey(),createProof(),hex2,hex6,hex7,hex8)
