import
  nimcrypto/[sha2, hash, utils]

proc concatArrays*(currentHeaderHash: array[32, byte],
                   newOptimisticHeader: array[32, byte],
                   newFinalizedHeader: array[32, byte],
                   newExecutionStateRoot: array[32, byte],
                   zerosSlotBuffer: array[24, byte],
                   currentSlot: array[8, byte],
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

proc hashHeaders*(currentHeaderHash: array[32, byte],
                  newOptimisticHeader: array[32, byte],
                  newFinalizedHeader: array[32, byte],
                  newExecutionStateRoot: array[32, byte],
                  zerosSlotBuffer: array[24, byte],
                  currentSlot: array[8, byte],
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
