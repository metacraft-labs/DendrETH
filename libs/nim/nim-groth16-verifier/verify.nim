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

proc makePairsAndVerify*(vk:VerificationKey, prf:Proof, preparedInputs:Point[G1]): bool {.wasmPragma.} =
  let aBPairing = pairing(prf.a, prf.b)
  let alphaBetaPairingP = pairing(vk.alpha, vk.beta)
  let preparedInputsGammaPairing = pairing(preparedInputs, vk.gamma)
  let proofCVkDeltaPairing = pairing(prf.c, vk.delta)
  let sum = alphaBetaPairingP * preparedInputsGammaPairing * proofCVkDeltaPairing;

  aBPairing == sum

