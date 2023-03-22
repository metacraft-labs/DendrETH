when defined(emcc):
  {.pragma: wasmPragma, cdecl, exportc, dynlib, codegenDecl: "__attribute__((used)) $# $#$#".}
else:
  {.pragma: wasmPragma, cdecl, exportc, dynlib.}
import
  ../../../../../libs/nim/nim-groth16-verifier/bncurve/groups,
  std/json,std/marshal

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

proc makePairsAndVerify*(vk: VerificationKey, prf: Proof, currentHeader: Header, newHeader: Header): bool {.wasmPragma.} =
  var preparedInputs = Input(data:vk.ic[0])

  echo $$preparedInputs.data
  preparedInputs.data = preparedInputs.data + (vk.ic[1] * currentHeader.head)
  echo $$preparedInputs.data
  preparedInputs.data = preparedInputs.data + (vk.ic[2] * currentHeader.tail)
  echo $$preparedInputs.data
  preparedInputs.data = preparedInputs.data + (vk.ic[3] * newHeader.head)
  echo $$preparedInputs.data
  preparedInputs.data = preparedInputs.data + (vk.ic[4] * newHeader.tail)
  echo $$preparedInputs.data
  let aBPairing = pairing(prf.a, prf.b)
  let alphaBetaPairingP = pairing(vk.alpha, vk.beta)
  let preparedInputsGammaPairing = pairing(preparedInputs.data, vk.gamma)
  let proofCVkDeltaPairing = pairing(prf.c, vk.delta)
  let sum = alphaBetaPairingP * preparedInputsGammaPairing * proofCVkDeltaPairing;

  aBPairing == sum

proc createVerificationKey*(): VerificationKey =
  let vk = parseFile("vkey.json")

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

  VerificationKey(alpha:vkAlpha1, beta:vkBeta2, gamma:vkGamma2, delta:vkDelta2, ic:icArr)


proc createProof*(): Proof =
  let proof = parseFile("proof.json")

  let a = Point[G1](x: FQ.fromString(proof["pi_a"][0].str), y: FQ.fromString(proof["pi_a"][1].str), z: FQ.fromString("1"))
  let b = Point[G2](x: FQ2(c0: FQ.fromString(proof["pi_b"][0][0].str),  c1: FQ.fromString(proof["pi_b"][0][1].str)), y: FQ2(c0: FQ.fromString(proof["pi_b"][1][0].str), c1: FQ.fromString(proof["pi_b"][1][1].str)), z: FQ2(c0: FQ.fromString("1"), c1: FQ.fromString("0")))
  let c = Point[G1](x: FQ.fromString(proof["pi_c"][0].str), y: FQ.fromString(proof["pi_c"][1].str), z: FQ.fromString("1"))

  Proof(a:a, b:b, c:c)

let currentHeader = Header(head: Fr.fromString("4574096041983537031700300285696450997075823807869821872596958510867517431425"),tail:Fr.fromString("5"))
let fakeNew = Header(head: Fr.fromString("4244096041983234532450300285696450997075823807869829732596958510867517431425"),tail:Fr.fromString("2"))

echo makePairsAndVerify(createVerificationKey(),createProof(),currentHeader,fakeNew)
