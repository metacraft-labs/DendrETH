import
  verify,
  std/[json,strformat], marshal
  #../../../vendor/nim-bncurve/bncurve


proc createVerificationKey*(path: string): VerificationKey =
  let vk = parseFile(path & "/verification_key.json")

  let vkAlpha1 = Point[G1](x: FQ.fromString(vk["vk_alpha_1"][0].str), y: FQ.fromString(vk["vk_alpha_1"][1].str), z: FQ.fromString("1"))
  let vkBeta2 = Point[G2](x: FQ2(c0: FQ.fromString(vk["vk_beta_2"][0][0].str),  c1: FQ.fromString(vk["vk_beta_2"][0][1].str)), y: FQ2(c0: FQ.fromString(vk["vk_beta_2"][1][0].str), c1: FQ.fromString(vk["vk_beta_2"][1][1].str)), z: FQ2(c0: FQ.fromString("1"), c1: FQ.fromString("0")))
  let vkGamma2 = Point[G2](x: FQ2(c0: FQ.fromString(vk["vk_gamma_2"][0][0].str),  c1: FQ.fromString(vk["vk_gamma_2"][0][1].str)), y: FQ2(c0: FQ.fromString(vk["vk_gamma_2"][1][0].str), c1: FQ.fromString(vk["vk_gamma_2"][1][1].str)), z: FQ2(c0: FQ.fromString("1"), c1: FQ.fromString("0")))
  let vkDelta2 = Point[G2](x: FQ2(c0: FQ.fromString(vk["vk_delta_2"][0][0].str),  c1: FQ.fromString(vk["vk_delta_2"][0][1].str)), y: FQ2(c0: FQ.fromString(vk["vk_delta_2"][1][0].str), c1: FQ.fromString(vk["vk_delta_2"][1][1].str)), z: FQ2(c0: FQ.fromString("1"), c1: FQ.fromString("0")))

  var icSeq: IC
  for el in vk["IC"]:
    let ic = Point[G1](x: FQ.fromString(el[0].str), y: FQ.fromString(el[1].str), z: FQ.fromString("1"))
    icSeq.add(ic)

  VerificationKey(alpha:vkAlpha1, beta:vkBeta2, gamma:vkGamma2, delta:vkDelta2, ic:icSeq)

proc createProof*(path: string, number: int): Proof =
  let proof = parseFile(path & fmt"/proof{number}.json");

  let a = Point[G1](x: FQ.fromString(proof["pi_a"][0].str), y: FQ.fromString(proof["pi_a"][1].str), z: FQ.fromString("1"))
  let b = Point[G2](x: FQ2(c0: FQ.fromString(proof["pi_b"][0][0].str),  c1: FQ.fromString(proof["pi_b"][0][1].str)), y: FQ2(c0: FQ.fromString(proof["pi_b"][1][0].str), c1: FQ.fromString(proof["pi_b"][1][1].str)), z: FQ2(c0: FQ.fromString("1"), c1: FQ.fromString("0")))
  let c = Point[G1](x: FQ.fromString(proof["pi_c"][0].str), y: FQ.fromString(proof["pi_c"][1].str), z: FQ.fromString("1"))

  Proof(a:a, b:b, c:c)

proc prepareInput*(path: string, number: int, ic:IC): Point[G1] =
  let public = parseFile(path & fmt"/public{number}.json")

  var preparedInputs = ic[0];
  for i in 0..(public.len-1):
    let pubInput = Fr.fromString(public[i].str)
    #echo "prepared input:"
    #echo $$preparedInputs
    #echo "ic:"
    #echo $$ic[i+1]
    echo "public:"
    echo public[i]
    echo "afterFromStringFunc:"
    echo $$pubInput
    echo ' '
    preparedInputs = preparedInputs + (ic[i+1] * pubInput)

  preparedInputs



proc mainVerify*(path: string, number: int): bool =

  let vk = createVerificationKey(path)
  let prf =createProof(path, number)
  let preparedInputs = prepareInput(path, number, vk.ic)
  #var t1: BNU256 = [1'u64,2'u64,3'u64,3'u64]
  makePairsAndVerify(vk, prf, preparedInputs)
