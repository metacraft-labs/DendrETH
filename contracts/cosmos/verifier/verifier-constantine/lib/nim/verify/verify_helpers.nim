import
  nimcrypto/[sha2, hash, utils],
  stew/byteutils

import
  constantine/math/arithmetic,
  constantine/math/config/curves,
  constantine/math/io/[io_ec, io_fields, io_bigints]

import ../../../../libs/nim/common

export hashHeaders

type
  Header* = object
    head*: Fr[BN254_Snarks]
    tail*: Fr[BN254_Snarks]

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
    firstNumInBytes[i] = bitArrayToByte([firstNumInBits[i*8],
                                         firstNumInBits[i*8+1],
                                         firstNumInBits[i*8+2],
                                         firstNumInBits[i*8+3],
                                         firstNumInBits[i*8+4],
                                         firstNumInBits[i*8+5],
                                         firstNumInBits[i*8+6],
                                         firstNumInBits[i*8+7]])

  var secondNumInBytes: array[32, byte]
  for i in 0..30:
    secondNumInBytes[i] = 0.byte
  secondNumInBytes[31] = bitArrayToByte([secondNumInBits[248],
                                         secondNumInBits[249],
                                         secondNumInBits[250],
                                         secondNumInBits[251],
                                         secondNumInBits[252],
                                         secondNumInBits[253],
                                         secondNumInBits[254],
                                         secondNumInBits[255]])

  var
    head: Fr[BN254_Snarks]
    tail: Fr[BN254_Snarks]
  head.fromHex(toHex(firstNumInBytes))
  tail.fromHex(toHex(secondNumInBytes))


  Header(head: head, tail: tail)
