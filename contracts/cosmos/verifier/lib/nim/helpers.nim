import
  ../../../../../libs/nim/nim-groth16-verifier/bncurve/groups,
  std/json,
  ../../../../../vendor/nimcrypto/nimcrypto/[sha2, hash, utils]



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
    head*: Fr
    tail*: Fr

  Input* = object
    data*: Point[G1]

proc createVerificationKey*(path: string): array[sizeof(VerificationKey),byte] =
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

  let vk1 = VerificationKey(alpha:vkAlpha1, beta:vkBeta2, gamma:vkGamma2, delta:vkDelta2, ic:icArr)
  result = cast[var array[sizeof(VerificationKey),byte]](vk1.unsafeAddr)

proc createProof*(path: string): array[sizeof(Proof),byte] =
  let proof = parseFile(path)

  let a = Point[G1](x: FQ.fromString(proof["pi_a"][0].str), y: FQ.fromString(proof["pi_a"][1].str), z: FQ.fromString("1"))
  let b = Point[G2](x: FQ2(c0: FQ.fromString(proof["pi_b"][0][0].str),  c1: FQ.fromString(proof["pi_b"][0][1].str)), y: FQ2(c0: FQ.fromString(proof["pi_b"][1][0].str), c1: FQ.fromString(proof["pi_b"][1][1].str)), z: FQ2(c0: FQ.fromString("1"), c1: FQ.fromString("0")))
  let c = Point[G1](x: FQ.fromString(proof["pi_c"][0].str), y: FQ.fromString(proof["pi_c"][1].str), z: FQ.fromString("1"))

  let prf = Proof(a:a, b:b, c:c)
  result = cast[var array[sizeof(Proof),byte]](prf.unsafeAddr)


proc createCurrentHeader*(path:string): array[sizeof(Header),byte]  =
  let public = parseFile(path)

  let currentHeader = Header(head: Fr.fromString(public[0].str),tail:Fr.fromString(public[1].str))
  result = cast[var array[sizeof(Header),byte]](currentHeader.unsafeAddr)


# proc createNewHeader*(path:string): array[sizeof(Header),byte] =
#   let public = parseFile(path)

#   let newHeader = Header(head: Fr.fromString(public[2].str),tail:Fr.fromString(public[3].str))
#   result = cast[var array[sizeof(Header),byte]](newHeader.unsafeAddr)

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

proc headerFromSeq(bigNum: seq): Header =
  var firstNumInBits: array[256, int]
  for i in 0..2:
    firstNumInBits[i] = 0

  var secondNumInBits: array[256, int]
  for i in 0..252:
    secondNumInBits[i] = 0

  for i in 0..30:
    var tempBitArray = decToBitArray(bigNum[i].int)
    for j in 0..7:
      firstNumInBits[i+j+3] = tempBitArray[j]

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
    head: BNU256
    tail: BNU256
  discard head.fromBytes(firstNumInBytes)
  discard tail.fromBytes(secondNumInBytes)
  # let newHeader = Header(head: head.FR, tail: tail.FR)
  # result = cast[var array[sizeof(Header),byte]](newHeader.unsafeAddr)
  Header(head: head.FR, tail: tail.FR)

proc createNewHeader*(path:string): array[sizeof(Header),byte] =
  # let public = parseFile(path)

  let hex = fromHex("0x51e177b2e6e99ae2b2179f54a471031622713a321d407b56fc3293c0d3d634bb")
  let head1 = headerFromSeq(hex)
  # let newHeader = Header(head: Fr.fromString(public[2].str),tail:Fr.fromString(public[3].str))
  result = cast[var array[sizeof(Header),byte]](head1.unsafeAddr)
