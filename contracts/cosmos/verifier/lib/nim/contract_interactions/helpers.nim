import
  bncurve/group_operations,
  std/json,
  stew/byteutils

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

proc getExpectedHeaderRoot*(path:string): array[32,byte] =
  let update = parseFile(path)
  let newOptimisticHeader = hexToByteArray[32](update["attested_header_root"].str)
  newOptimisticHeader

proc getExpectedFinalizedRoot*(path:string): array[32,byte] =
  let update = parseFile(path)
  let newFinalizedHeader = hexToByteArray[32](update["finalized_header_root"].str)
  newFinalizedHeader

proc getExpectedExecutionStateRoot*(path:string): array[32,byte] =
  let update = parseFile(path)
  let newExecStateRoot = hexToByteArray[32](update["finalized_execution_state_root"].str)
  newExecStateRoot

