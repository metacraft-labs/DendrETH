when defined(emcc):
  {.pragma: wasmPragma, cdecl, exportc, dynlib, codegenDecl: "__attribute__((used)) $# $#$#".}
else:
  {.pragma: wasmPragma, cdecl, exportc, dynlib.}
import
  verify_helpers

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

proc makePairsAndVerify*(vk: ref VerificationKey,
                         prf: ref Proof,
                         currentHeaderHash: var array[32, byte],
                         newOptimisticHeader: array[32, byte],
                         newFinalizedHeader: array[32, byte],
                         newExecutionStateRoot: array[32, byte],
                         currentSlot: array[8, byte],
                         domain: array[32, byte]
                        ): bool {.wasmPragma.} =
  var reverseSlot: array[8, byte]
  for i in 0..7:
    reverseSlot[i] = currentSlot[7-i]
  var zerosSlotBuffer: array[24, byte]
  for i in 0..23:
    zerosSlotBuffer[i] = 0
  let sha256ofHashes = hashHeaders(currentHeaderHash, newOptimisticHeader, newFinalizedHeader, newExecutionStateRoot, zerosSlotBuffer, reverseSlot, domain)
  let header = headerFromSeq(@sha256ofHashes)

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
