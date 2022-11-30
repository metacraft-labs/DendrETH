when defined(emcc):
  {.pragma: wasmPragma, cdecl, exportc, dynlib, codegenDecl: "__attribute__((used)) $# $#$#".}
else:
  {.pragma: wasmPragma, cdecl, exportc, dynlib.}
import
  ../../../../../libs/nim/nim-groth16-verifier/bncurve/groups

export groups

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

proc makePairsAndVerify*(vk: ref VerificationKey, prf: ref Proof, currentHeader: ref Header, newHeader: ref Header): bool {.wasmPragma.} =
  var preparedInputs = Input(data:vk[].ic[0])
  preparedInputs.data = preparedInputs.data + (vk[].ic[1] * currentHeader[].head)
  preparedInputs.data = preparedInputs.data + (vk[].ic[2] * currentHeader[].tail)
  preparedInputs.data = preparedInputs.data + (vk[].ic[3] * newHeader[].head)
  preparedInputs.data = preparedInputs.data + (vk[].ic[4] * newHeader[].tail)

  let aBPairing = pairing(prf[].a, prf[].b)
  let alphaBetaPairingP = pairing(vk[].alpha, vk[].beta)
  let preparedInputsGammaPairing = pairing(preparedInputs.data, vk[].gamma)
  let proofCVkDeltaPairing = pairing(prf[].c, vk[].delta)
  let sum = alphaBetaPairingP * preparedInputsGammaPairing * proofCVkDeltaPairing;

  aBPairing == sum
