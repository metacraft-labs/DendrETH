import
  std/json,
  stew/byteutils

import
  constantine/math/pairings/pairings_bn,
  constantine/math/elliptic/[ec_shortweierstrass_affine, ec_shortweierstrass_projective],
  constantine/math/io/[io_ec, io_fields, io_bigints],
  constantine/math/elliptic/ec_scalar_mul,
  constantine/math/config/type_bigint,
  constantine/math/arithmetic,
  constantine/math/config/curves,
  constantine/math/io/io_extfields

type
  IC* = array[3, ECP_ShortW_Aff[Fp[BN254_Snarks], G1]]

  VerificationKey* = object
    alpha*: ECP_ShortW_Aff[Fp[BN254_Snarks], G1]
    beta*, gamma*, delta*: ECP_ShortW_Aff[Fp2[BN254_Snarks], G2]
    ic*: IC

  Proof* = object
    a*, c*: ECP_ShortW_Aff[Fp[BN254_Snarks], G1]
    b*: ECP_ShortW_Aff[Fp2[BN254_Snarks], G2]

  Header* = object
    head*: Fr[BN254_Snarks]
    tail*: Fr[BN254_Snarks]

  Input* = object
    data*: ECP_ShortW_Aff[Fp[BN254_Snarks], G1]

proc createVerificationKey*(path: string): array[sizeof(VerificationKey),byte] =
  let vk = parseFile(path)

  let alpha0 = BigInt[255].fromDecimal(vk["vk_alpha_1"][0].str).toHex()
  let alpha1 = BigInt[255].fromDecimal(vk["vk_alpha_1"][1].str).toHex()
  let vkAlpha1 = ECP_ShortW_Aff[Fp[BN254_Snarks], G1].fromHex(alpha0, alpha1)

  let beta00 = BigInt[255].fromDecimal(vk["vk_beta_2"][0][0].str).toHex()
  let beta01 = BigInt[255].fromDecimal(vk["vk_beta_2"][0][1].str).toHex()
  let beta10 = BigInt[255].fromDecimal(vk["vk_beta_2"][1][0].str).toHex()
  let beta11 = BigInt[255].fromDecimal(vk["vk_beta_2"][1][1].str).toHex()
  let vkBeta2 = ECP_ShortW_Aff[Fp2[BN254_Snarks], G2].fromHex(beta00, beta01, beta10, beta11)

  let gamma00 = BigInt[255].fromDecimal(vk["vk_gamma_2"][0][0].str).toHex()
  let gamma01 = BigInt[255].fromDecimal(vk["vk_gamma_2"][0][1].str).toHex()
  let gamma10 = BigInt[255].fromDecimal(vk["vk_gamma_2"][1][0].str).toHex()
  let gamma11 = BigInt[255].fromDecimal(vk["vk_gamma_2"][1][1].str).toHex()
  let vkGamma2 = ECP_ShortW_Aff[Fp2[BN254_Snarks], G2].fromHex(gamma00, gamma01, gamma10, gamma11)

  let delta00 = BigInt[255].fromDecimal(vk["vk_delta_2"][0][0].str).toHex()
  let delta01 = BigInt[255].fromDecimal(vk["vk_delta_2"][0][1].str).toHex()
  let delta10 = BigInt[255].fromDecimal(vk["vk_delta_2"][1][0].str).toHex()
  let delta11 = BigInt[255].fromDecimal(vk["vk_delta_2"][1][1].str).toHex()
  let vkDelta2 = ECP_ShortW_Aff[Fp2[BN254_Snarks], G2].fromHex(delta00, delta01, delta10, delta11)

  var icArr: IC
  var counter = 0
  for el in vk["IC"]:
    let ic0 = BigInt[255].fromDecimal(el[0].str).toHex()
    let ic1 = BigInt[255].fromDecimal(el[1].str).toHex()
    let ic = ECP_ShortW_Aff[Fp[BN254_Snarks], G1].fromHex(ic0, ic1)
    icArr[counter] = ic
    counter+=1

  let vk1 = VerificationKey(alpha:vkAlpha1, beta:vkBeta2, gamma:vkGamma2, delta:vkDelta2, ic:icArr)
  result = cast[var array[sizeof(VerificationKey),byte]](vk1.unsafeAddr)

proc createProof*(path: string): array[sizeof(Proof),byte] =
  let proof = parseFile(path)

  let a0 = BigInt[255].fromDecimal(proof["pi_a"][0].str).toHex()
  let a1 = BigInt[255].fromDecimal(proof["pi_a"][1].str).toHex()
  let a = ECP_ShortW_Aff[Fp[BN254_Snarks], G1].fromHex(a0, a1)

  let b00 = BigInt[255].fromDecimal(proof["pi_b"][0][0].str).toHex()
  let b01 = BigInt[255].fromDecimal(proof["pi_b"][0][1].str).toHex()
  let b10 = BigInt[255].fromDecimal(proof["pi_b"][1][0].str).toHex()
  let b11 = BigInt[255].fromDecimal(proof["pi_b"][1][1].str).toHex()
  let b = ECP_ShortW_Aff[Fp2[BN254_Snarks], G2].fromHex(b00, b01, b10, b11)

  let c0 = BigInt[255].fromDecimal(proof["pi_c"][0].str).toHex()
  let c1 = BigInt[255].fromDecimal(proof["pi_c"][1].str).toHex()
  let c = ECP_ShortW_Aff[Fp[BN254_Snarks], G1].fromHex(c0, c1)

  let prf = Proof(a:a, b:b, c:c)
  result = cast[var array[sizeof(Proof),byte]](prf.unsafeAddr)

proc getExpectedHeaderRoot*(path:string): array[32,byte] =
  let update = parseFile(path)
  let newOptimisticHeader = hexToByteArray[32](update["attestedHeaderRoot"].str)
  newOptimisticHeader

proc getExpectedFinalizedRoot*(path:string): array[32,byte] =
  let update = parseFile(path)
  let newFinalizedHeader = hexToByteArray[32](update["finalizedHeaderRoot"].str)
  newFinalizedHeader

proc getExpectedExecutionStateRoot*(path:string): array[32,byte] =
  let update = parseFile(path)
  let newExecStateRoot = hexToByteArray[32](update["finalizedExecutionStateRoot"].str)
  newExecStateRoot

proc getExpectedSlot*(path:string): JsonNode =
  let update = parseFile(path)
  let newSlot = update["attestedHeaderSlot"]
  newSlot
