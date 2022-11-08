# Nim Barreto-Naehrig pairing-friendly elliptic curve implementation
# Copyright (c) 2018 Status Research & Development GmbH
# Licensed under either of
#  * Apache License, version 2.0, ([LICENSE-APACHE](LICENSE-APACHE))
#  * MIT license ([LICENSE-MIT](LICENSE-MIT))
# at your option.
# This file may not be copied, modified, or distributed except according to
# those terms.
import options
import fq6, fq2, fp, arith

{.deadCodeElim: on.}

const frobeniusCoeffsC1: array[4, FQ2] = [
  FQ2.one(),
  FQ2(
    c0: FQ([12653890742059813127'u64, 14585784200204367754'u64,
            1278438861261381767'u64, 212598772761311868'u64]),
    c1: FQ([11683091849979440498'u64, 14992204589386555739'u64,
            15866167890766973222'u64, 1200023580730561873'u64])
  ),
  FQ2(
    c0: FQ([14595462726357228530'u64, 17349508522658994025'u64,
            1017833795229664280'u64, 299787779797702374'u64]),
    c1: FQ.zero()
  ),
  FQ2(
    c0: FQ([3914496794763385213'u64, 790120733010914719'u64,
            7322192392869644725'u64, 581366264293887267'u64]),
    c1: FQ([12817045492518885689'u64, 4440270538777280383'u64,
            11178533038884588256'u64, 2767537931541304486'u64])
  )
]

type
  FQ12* = object
    c0*: FQ6
    c1*: FQ6

proc init*(c0, c1: FQ6): FQ12 {.inline, noinit.} =
  result.c0 = c0
  result.c1 = c1

proc zero*(t: typedesc[FQ12]): FQ12 {.inline, noinit.} =
  result.c0 = FQ6.zero()
  result.c1 = FQ6.zero()

proc one*(t: typedesc[FQ12]): FQ12 {.inline, noinit.} =
  result.c0 = FQ6.one()
  result.c1 = FQ6.zero()

proc isZero*(x: FQ12): bool {.inline, noinit.} =
  result = (x.c0.isZero() and x.c1.isZero())

proc squared*(x: FQ12): FQ12 {.inline, noinit.} =
  let ab = x.c0 * x.c1
  result.c0 = (x.c1.mulByNonresidue() + x.c0) * (x.c0 + x.c1) - ab -
              ab.mulByNonresidue()
  result.c1 = ab + ab

proc inverse*(x: FQ12): Option[FQ12] {.inline, noinit.} =
  let opt = (x.c0.squared() - (x.c1.squared().mulByNonresidue())).inverse()
  if isSome(opt):
    let tmp = opt.get()
    result = some[FQ12](FQ12(c0: x.c0 * tmp, c1: -(x.c1 * tmp)))
  else:
    result = none[FQ12]()

proc `+`*(x, y: FQ12): FQ12 {.noinit, inline.} =
  ## Return result of ``x + y``.
  result.c0 = x.c0 + y.c0
  result.c1 = x.c1 + y.c1

proc `+=`*(x: var FQ12, y: FQ12) {.noinit, inline.} =
  ## Perform inplace addition ``x = x + y``.
  x.c0 += y.c0
  x.c1 += y.c1

proc `-`*(x, y: FQ12): FQ12 {.noinit, inline.} =
  ## Return result of ``x - y``.
  result.c0 = x.c0 - y.c0
  result.c1 = x.c1 - y.c1

proc `-=`*(x: var FQ12, y: FQ12) {.noinit, inline.} =
  ## Perform inplace substraction ``x = x - y``.
  x.c0 -= y.c0
  x.c1 -= y.c1

proc `*`*(x, y: FQ12): FQ12 {.noinit, inline.} =
  ## Return result of ``x * y``.
  let aa = x.c0 * y.c0
  let bb = x.c1 * y.c1
  result.c0 = bb.mulByNonresidue() + aa
  result.c1 = (x.c0 + x.c1) * (y.c0 + y.c1) - aa - bb

proc `*=`*(x: var FQ12, y: FQ12) {.noinit, inline.} =
  ## Perform inplace multiplication ``x = x * y``.
  let aa = x.c0 * y.c0
  let bb = x.c1 * y.c1
  let cc = x.c0 + x.c1
  x.c0 = bb.mulByNonresidue() + aa
  x.c1 = cc * (y.c0 + y.c1) - aa - bb

proc `-`*(x: FQ12): FQ12 {.noinit, inline.} =
  ## Negotiation of ``x``.
  result.c0 = -x.c0
  result.c1 = -x.c1

proc pow*(x: FQ12, by: BNU256): FQ12 {.noinit.} =
  result = FQ12.one()
  for i in by.bits():
    result = result.squared()
    if i:
      result *= x

proc pow*(x: FQ12, by: FR): FQ12 {.inline, noinit.} =
  result = pow(x, BNU256.into(by))

proc frobeniusMap*(x: FQ12, power: uint64): FQ12 =
  result.c0 = x.c0.frobeniusMap(power)
  result.c1 = x.c1.frobeniusMap(power).scale(frobeniusCoeffsC1[power mod 12])

proc unitaryInverse*(x: FQ12): FQ12 =
  result.c0 = x.c0
  result.c1 = -x.c1

proc cyclotomicSquared*(x: FQ12): FQ12 =
  var z0 = x.c0.c0
  var z4 = x.c0.c1
  var z3 = x.c0.c2
  var z2 = x.c1.c0
  var z1 = x.c1.c1
  var z5 = x.c1.c2

  var tmp = z0 * z1
  let t0 = (z0 + z1) * (z1.mulByNonresidue() + z0) - tmp - tmp.mulByNonresidue()
  let t1 = tmp + tmp

  tmp = z2 * z3;
  let t2 = (z2 + z3) * (z3.mulByNonresidue() + z2) - tmp - tmp.mulByNonresidue()
  let t3 = tmp + tmp

  tmp = z4 * z5;
  let t4 = (z4 + z5) * (z5.mulByNonresidue() + z4) - tmp - tmp.mulByNonresidue()
  let t5 = tmp + tmp

  z0 = t0 - z0
  z0 = z0 + z0
  z0 = z0 + t0

  z1 = t1 + z1
  z1 = z1 + z1
  z1 = z1 + t1

  tmp = t5.mulByNonresidue()
  z2 = tmp + z2
  z2 = z2 + z2
  z2 = z2 + tmp

  z3 = t4 - z3
  z3 = z3 + z3
  z3 = z3 + t4

  z4 = t2 - z4
  z4 = z4 + z4
  z4 = z4 + t2

  z5 = t3 + z5
  z5 = z5 + z5
  z5 = z5 + t3

  result.c0 = init(z0, z4, z3)
  result.c1 = init(z2, z1, z5)

proc cyclotomicPow*(x: FQ12, by: BNU256): FQ12 =
  result = FQ12.one()
  var foundOne = false

  for i in by.bits():
    if foundOne:
      result = result.cyclotomicSquared()
    if i:
      foundOne = true
      result = x * result

proc expByNegZ*(x: FQ12): FQ12 =
  let uconst = BNU256([4965661367192848881'u64, 0'u64, 0'u64, 0'u64])
  result = x.cyclotomicPow(uconst).unitaryInverse()

proc finalExpFirstChunk*(x: FQ12): Option[FQ12] =
  let opt = x.inverse()
  if isSome(opt):
    let b = opt.get()
    let a = x.unitaryInverse()
    let c = a * b
    let d = c.frobeniusMap(2)
    result = some[FQ12](d * c)
  else:
    result = none[FQ12]()

proc finalExpLastChunk*(x: FQ12): FQ12 =
  let a = x.expByNegZ()
  let b = a.cyclotomicSquared()
  let c = b.cyclotomicSquared()
  let d = c * b

  let e = d.expByNegZ()
  let f = e.cyclotomicSquared()
  let g = f.expByNegZ()
  let h = d.unitaryInverse()
  let i = g.unitaryInverse()

  let j = i * e
  let k = j * h
  let ll = k * b
  let m = k * e
  let n = x * m

  let o = ll.frobeniusMap(1)
  let p = o * n

  let q = k.frobeniusMap(2)
  let r = q * p

  let s = x.unitaryInverse()
  let t = s * ll
  let u = t.frobeniusMap(3)
  let v = u * r

  result = v

proc finalExponentiation*(x: FQ12): Option[FQ12] =
  let opt = x.finalExpFirstChunk()
  if opt.isSome():
    result = some[FQ12](opt.get().finalExpLastChunk())
  else:
    result = none[FQ12]()

proc mulBy024*(x: FQ12, ell0, ellvw, ellvv: FQ2): FQ12 =
  var
    z0, z1, z2, z3, z4, z5: FQ2
    x0, x2, x4, d0, d2, d4: FQ2
    s0, s1, t0, t1, t2, t3, t4: FQ2

  z0 = x.c0.c0
  z1 = x.c0.c1
  z2 = x.c0.c2
  z3 = x.c1.c0
  z4 = x.c1.c1
  z5 = x.c1.c2

  x0 = ell0
  x2 = ellvv
  x4 = ellvw

  d0 = z0 * x0
  d2 = z2 * x2
  d4 = z4 * x4
  t2 = z0 + z4
  t1 = z0 + z2
  s0 = z1 + z3 + z5

  s1 = z1 * x2
  t3 = s1 + d4
  t4 = t3.mulByNonresidue() + d0
  z0 = t4

  t3 = z5 * x4
  s1 = s1 + t3
  t3 = t3 + d2
  t4 = t3.mulByNonresidue()
  t3 = z1 * x0
  s1 = s1 + t3
  t4 = t4 + t3
  z1 = t4

  t0 = x0 + x2
  t3 = t1 * t0 - d0 - d2
  t4 = z3 * x4
  s1 = s1 + t4
  t3 = t3 + t4

  t0 = z2 + z4
  z2 = t3

  t1 = x2 + x4
  t3 = t0 * t1 - d2 - d4
  t4 = t3.mulByNonresidue()
  t3 = z3 * x0
  s1 = s1 + t3
  t4 = t4 + t3
  z3 = t4

  t3 = z5 * x2
  s1 = s1 + t3
  t4 = t3.mulByNonresidue()
  t0 = x0 + x4
  t3 = t2 * t0 - d0 - d4
  t4 = t4 + t3
  z4 = t4

  t0 = x0 + x2 + x4
  t3 = s0 * t0 - s1
  z5 = t3

  result.c0 = init(z0, z1, z2)
  result.c1 = init(z3, z4, z5)
