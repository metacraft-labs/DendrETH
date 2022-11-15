when defined(emcc):
  {.pragma: wasmPragma, cdecl, exportc, dynlib, codegenDecl: "__attribute__((used)) $# $#$#".}
else:
  {.pragma: wasmPragma, cdecl, exportc, dynlib.}
import
  bncurve/groups

export groups

type
  IC* = seq[Point[G1]]

  VerificationKey* = object
    alpha*: Point[G1]
    beta*, gamma*, delta*: Point[G2]
    ic*: IC

  Proof* = object
    a*, c*: Point[G1]
    b*: Point[G2]

  HeaderHash* = object
    a*: Point[G1]


# get new header and new proof
# create input with current header, new header and IC of VerificationKey
# check if VK+Proof+input = correct
# save new header as current header
#proc prepareHeaders*(currentHeader:HeaderHash,newHeader:HeaderHash,ic:IC)

proc testVerify*(vk:int, prf:int, input:int): int {.wasmPragma.} =
  vk - prf - input


#  # result

proc makePairsAndVerify*(vk:VerificationKey, prf:Proof, preparedInputs:Point[G1]): bool {.wasmPragma.} =
  let aBPairing = pairing(prf.a, prf.b)
  let alphaBetaPairingP = pairing(vk.alpha, vk.beta)
  let preparedInputsGammaPairing = pairing(preparedInputs, vk.gamma)
  let proofCVkDeltaPairing = pairing(prf.c, vk.delta)
  let sum = alphaBetaPairingP * preparedInputsGammaPairing * proofCVkDeltaPairing;
  #echo testVerify(1,1,2)
  aBPairing == sum

proc testproc*(a:int,b:int): int {.wasmPragma.} =
  return a*b

proc testproc2*(a:int, b:int): bool {.wasmPragma.} =
  return a==b


