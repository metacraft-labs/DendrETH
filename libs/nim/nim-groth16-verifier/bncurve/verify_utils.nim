# import
  # bncurve/groups
import std/options
import ../../../vendor/nim/lib/pure/endians
from nimcrypto/utils import fromHex,toHex
import bncurve/fields
#export bncurve/fields

type
  IC* = seq[Point[G1]]

  VerificationKey* = object
    alpha*: Point[G1]
    beta*, gamma*, delta*: Point[G2]
    ic*: IC

  Proof* = object
    a*, c*: Point[G1]
    b*: Point[G2]

  G1* = object
  G2* = object

  Point*[T: G1|G2] = object
    when T is G1:
      x*, y*, z*: FQ
    else:
      x*, y*, z*: FQ2

  AffinePoint*[T: G1|G2] = object
    when T is G1:
      x*, y*: FQ
    else:
      x*, y*: FQ2

  EllCoeffs* = object
    ell_0*: FQ2
    ell_vw*: FQ2
    ell_vv*: FQ2

  G2Precomp* = object
    q*: AffinePoint[G2]
    coeffs*: seq[EllCoeffs]

  FQ12* = object
    c0*: FQ6
    c1*: FQ6

  FQ6* = object
    c0*: FQ2
    c1*: FQ2
    c2*: FQ2

  FQ2* = object
    c0*: FQ
    c1*: FQ

proc pairing*(p: Point[G1], q: Point[G2]): FQ12 {.noinit, inline.} =
  result = FQ12.one()
  var optp = p.toAffine()
  var optq = q.toAffine()
  if optp.isSome() and optq.isSome():
    let pc = optq.get().precompute()
    let ores = finalExponentiation(pc.millerLoop(optp.get()))
    if ores.isSome():
      result = ores.get()


