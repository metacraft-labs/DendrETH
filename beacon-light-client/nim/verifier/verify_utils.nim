import
  bncurve/group_operations,
  nimcrypto/[sha2, hash, utils]
export
  group_operations

type
  Header* = object
    head*: FR
    tail*: FR

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
