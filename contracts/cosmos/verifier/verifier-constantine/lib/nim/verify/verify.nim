when defined(emcc):
  {.pragma: wasmPragma, cdecl, exportc, dynlib, codegenDecl: "__attribute__((used)) $# $#$#".}
else:
  {.pragma: wasmPragma, cdecl, exportc, dynlib.}
import
  constantine/math/pairings/pairings_bn,
  constantine/math/elliptic/[ec_shortweierstrass_affine, ec_shortweierstrass_projective],
  constantine/math/io/[io_fields, io_bigints],
  constantine/math/elliptic/ec_scalar_mul,
  constantine/math/config/type_bigint,
  constantine/math/arithmetic,
  constantine/math/config/curves,
  constantine/math/io/io_extfields

import
    verify_helpers

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


proc makePairsAndVerify*(vk: VerificationKey,
                         prf: Proof,
                         currentHeaderHash: var array[32, byte],
                         newOptimisticHeader: var array[32, byte],
                         newFinalizedHeader: array[32, byte],
                         newExecutionStateRoot: array[32, byte],
                         currentSlot: array[8, byte],
                         domain: array[32, byte]): bool {.wasmPragma.} =
  var reverseSlot: array[8, byte]
  for i in 0..7:
    reverseSlot[i] = currentSlot[7-i]
  var zerosSlotBuffer: array[24, byte]
  for i in 0..23:
    zerosSlotBuffer[i] = 0

  let sha256ofHashes = hashHeaders(currentHeaderHash, newOptimisticHeader,
                                   newFinalizedHeader, newExecutionStateRoot,
                                   zerosSlotBuffer, reverseSlot, domain)
  let header = headerFromSeq(@sha256ofHashes)

  var preparedInputs:Input
  var ic0Prj: ECP_ShortW_Prj[Fp[BN254_Snarks], G1]
  var ic1Prj: ECP_ShortW_Prj[Fp[BN254_Snarks], G1]
  var ic2Prj: ECP_ShortW_Prj[Fp[BN254_Snarks], G1]
  fromAffine(ic0Prj, vk.ic[0])
  fromAffine(ic1Prj, vk.ic[1])
  fromAffine(ic2Prj, vk.ic[2])
  scalarMul(ic1Prj,toBig(header.head))
  scalarMul(ic2Prj,toBig(header.tail))
  ic0Prj += ic1Prj
  ic0Prj += ic2Prj
  affine(preparedInputs.data,ic0Prj)

  var aBPairing: Fp12[BN254_Snarks]
  pairing_bn[BN254_Snarks](aBPairing,prf.a,prf.b)
  var alphaBetaPairing: Fp12[BN254_Snarks]
  pairing_bn[BN254_Snarks](alphaBetaPairing,vk.alpha,vk.beta)
  var preparedInputsGammaPairing: Fp12[BN254_Snarks]
  pairing_bn[BN254_Snarks](preparedInputsGammaPairing,preparedInputs.data,vk.gamma)
  var proofCVkDeltaPairing: Fp12[BN254_Snarks]
  pairing_bn[BN254_Snarks](proofCVkDeltaPairing,prf.c,vk.delta)

  var sum:Fp12[BN254_Snarks]
  prod(sum, alphaBetaPairing, preparedInputsGammaPairing)
  prod(sum, sum, proofCVkDeltaPairing)

  if (sum == aBPairing).bool:
    currentHeaderHash = newOptimisticHeader
    return true
  else:
    return false
  (sum == aBPairing).bool
