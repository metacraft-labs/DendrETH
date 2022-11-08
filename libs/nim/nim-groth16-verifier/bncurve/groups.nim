# Nim Barreto-Naehrig pairing-friendly elliptic curve implementation
# Copyright (c) 2018 Status Research & Development GmbH
# Licensed under either of
#  * Apache License, version 2.0, ([LICENSE-APACHE](LICENSE-APACHE))
#  * MIT license ([LICENSE-MIT](LICENSE-MIT))
# at your option.
# This file may not be copied, modified, or distributed except according to
# those terms.
import fields, arith, options
export fields, arith, options


{.deadCodeElim: on.}

type
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

const
  G1One = Point[G1](
    x: FQ.one(),
    y: FQ([0xa6ba871b8b1e1b3a'u64, 0x14f1d651eb8e167b'u64,
           0xccdd46def0f28c58'u64, 0x1c14ef83340fbe5e'u64]),
    z: FQ.one()
  )

  G1B = FQ([0x7a17caa950ad28d7'u64, 0x1f6ac17ae15521b9'u64,
            0x334bea4e696bd284'u64, 0x2a1f6744ce179d8e'u64])

  G2One = Point[G2](
    x: FQ2(
      c0: FQ([0x8e83b5d102bc2026'u64, 0xdceb1935497b0172'u64,
              0xfbb8264797811adf'u64, 0x19573841af96503b'u64]),
      c1: FQ([0xafb4737da84c6140'u64, 0x6043dd5a5802d8c4'u64,
              0x09e950fc52a02f86'u64, 0x14fef0833aea7b6b'u64])
    ),
    y: FQ2(
      c0: FQ([0x619dfa9d886be9f6'u64, 0xfe7fd297f59e9b78'u64,
              0xff9e1a62231b7dfe'u64, 0x28fd7eebae9e4206'u64]),
      c1: FQ([0x64095b56c71856ee'u64, 0xdc57f922327d3cbb'u64,
              0x55f935be33351076'u64, 0x0da4a0e693fd6482'u64])
    ),
    z: FQ2.one()
  )

  G2B = FQ2(
    c0: FQ([0x3bf938e377b802a8'u64, 0x020b1b273633535d'u64,
            0x26b7edf049755260'u64, 0x2514c6324384a86d'u64]),
    c1: FQ([0x38e7ecccd1dcff67'u64, 0x65f0b37d93ce0d3e'u64,
            0xd749d0dd22ac00aa'u64, 0x0141b9ce4a688d4d'u64])
  )

  AteLoopCount = BNU256([
    0x9d797039be763ba8'u64, 0x0000000000000001'u64,
    0x0000000000000000'u64, 0x0000000000000000'u64
  ])

  TwoInv = FQ([
    9781510331150239090'u64, 15059239858463337189'u64,
    10331104244869713732'u64, 2249375503248834476'u64
  ])

  Twist = FQ2NonResidue

  TwistMulByQx = FQ2(
    c0: FQ([
      13075984984163199792'u64, 3782902503040509012'u64,
      8791150885551868305'u64, 1825854335138010348'u64
    ]),
    c1: FQ([
      7963664994991228759'u64, 12257807996192067905'u64,
      13179524609921305146'u64, 2767831111890561987'u64
    ])
  )

  TwistMulByQy = FQ2(
    c0: FQ([
      16482010305593259561'u64, 13488546290961988299'u64,
      3578621962720924518'u64, 2681173117283399901'u64
    ]),
    c1: FQ([
      11661927080404088775'u64, 553939530661941723'u64,
      7860678177968807019'u64, 3208568454732775116'u64
    ])
  )

proc one*[T: G1|G2](t: typedesc[T]): Point[T] {.inline, noinit.} =
  when T is G1:
    result = G1One
  else:
    result = G2One

# proc one*(t: typedesc[Gt]): Gt {.inline, noinit.} =
#   result = FQ12.one()

proc name*[T: G1|G2](t: typedesc[T]): string {.inline, noinit.} =
  when T is G1:
    result = "G1"
  else:
    result = "G2"

proc coeff*(t: typedesc[G1]): FQ {.inline, noinit.} =
  result = G1B

proc coeff*(t: typedesc[G2]): FQ2 {.inline, noinit.} =
  result = G2B

proc zero*[T: G1|G2](t: typedesc[T]): Point[T] {.inline, noinit.} =
  when T is G1:
    result.x = FQ.zero()
    result.y = FQ.one()
    result.z = FQ.zero()
  else:
    result.x = FQ2.zero()
    result.y = FQ2.one()
    result.z = FQ2.zero()

proc isZero*[T: G1|G2](p: Point[T]): bool {.inline, noinit.} =
  result = p.z.isZero()

proc double*[T: G1|G2](p: Point[T]): Point[T] {.noinit.} =
  let a = p.x.squared()
  let b = p.y.squared()
  let c = b.squared()
  var d = (p.x + b).squared() - a - c
  d = d + d
  let e = a + a + a
  let f = e.squared()
  let x3 = f - (d + d)
  var eightc = c + c
  eightc = eightc + eightc
  eightc = eightc + eightc
  let y1z1 = p.y * p.z
  result.x = x3
  result.y = e * (d - x3) - eightc
  result.z = y1z1 + y1z1

proc `*`*[T: G1|G2](p: Point[T], by: FR): Point[T] =
  result = T.zero()
  var foundOne = false
  for i in BNU256.into(by).bits():
    if foundOne:
      result = result.double()
    if i:
      foundOne = true
      result = result + p

proc `+`*[T: G1|G2](p1, p2: Point[T]): Point[T] {.noinit.} =
  if p1.isZero():
    return p2
  if p2.isZero():
    return p1

  let z1squared = p1.z.squared()
  let z2squared = p2.z.squared()
  let u1 = p1.x * z2squared
  let u2 = p2.x * z1squared
  let z1cubed = p1.z * z1squared
  let z2cubed = p2.z * z2squared
  let s1 = p1.y * z2cubed
  let s2 = p2.y * z1cubed

  if u1 == u2 and s1 == s2:
    result = p1.double()
  else:
    let h = u2 - u1
    let s2minuss1 = s2 - s1
    let i = (h + h).squared()
    let j = h * i
    let r = s2minuss1 + s2minuss1
    let v = u1 * i
    let s1j = s1 * j
    let x3 = r.squared() - j - (v + v)
    result.x = x3
    result.y = r * (v - x3) - (s1j + s1j)
    result.z = ((p1.z + p2.z).squared() - z1squared - z2squared) * h

proc `-`*[T: G1|G2](p: Point[T]): Point[T] {.inline, noinit.} =
  if p.isZero():
    return p
  else:
    result.x = p.x
    result.y = -p.y
    result.z = p.z

proc `-`*[T: G1|G2](p: AffinePoint[T]): AffinePoint[T] {.inline, noinit.} =
  result.x = p.x
  result.y = -p.y

proc `-`*[T: G1|G2](p1, p2: Point[T]): Point[T] {.inline, noinit.} =
  result = p1 + (-p2)

proc `==`*[T: G1|G2](p1, p2: Point[T]): bool =
  if p1.isZero():
    return p2.isZero()
  if p2.isZero():
    return false
  let z1squared = p1.z.squared()
  let z2squared = p2.z.squared()
  if (p1.x * z2squared) != (p2.x * z1squared):
    return false
  let z1cubed = p1.z * z1squared
  let z2cubed = p2.z * z2squared
  if (p1.y * z2cubed) != (p2.y * z1cubed):
    return false
  return true

proc toJacobian*[T: G1|G2](p: AffinePoint[T]): Point[T] {.inline, noinit.} =
  ## Convert affine coordinates' point ``p`` to point.
  result.x = p.x
  result.y = p.y
  when T is G1:
    result.z = FQ.one()
  else:
    result.z = FQ2.one()

proc toAffine*[T: G1|G2](p: Point[T]): Option[AffinePoint[T]] =
  ## Attempt to convert point ``p`` to affine coordinates.
  when T is G1:
    var fone = FQ.one()
  else:
    var fone = FQ2.one()

  if p.z.isZero():
    result = none[AffinePoint[T]]()
  elif p.z == fone:
    result = some[AffinePoint[T]](AffinePoint[T](x: p.x, y: p.y))
  else:
    let ozinv = p.z.inverse()
    if isSome(ozinv):
      let zinv = ozinv.get()
      var zinvsquared = zinv.squared()
      result = some[AffinePoint[T]](
        AffinePoint[T](
          x: p.x * zinvsquared,
          y: p.y * (zinvsquared * zinv)
        )
      )
    else:
      result = none[AffinePoint[T]]()

proc isOnCurve*[T: G1|G2](p: AffinePoint[T]): bool =
  when T is G1:
    result = (p.y.squared() == (p.x.squared() * p.x) + G1B)
  else:
    result = (p.y.squared() == (p.x.squared() * p.x) + G2B)

proc mulByQ(p: AffinePoint[G2]): AffinePoint[G2] =
  result.x = TwistMulByQx * p.x.frobeniusMap(1)
  result.y = TwistMulByQy * p.y.frobeniusMap(1)

proc mixedAdditionStepForFlippedML(p: var Point[G2],
                                   base: AffinePoint[G2]): EllCoeffs =
  let d = p.x - p.z * base.x
  let e = p.y - p.z * base.y
  let f = d.squared()
  let g = e.squared()
  let h = d * f
  let i = p.x * f
  let j = p.z * g + h - (i + i)

  p.x = d * j
  p.y = e * (i - j) - h * p.y
  p.z = p.z * h

  result.ell_0 = Twist * (e * base.x - d * base.y)
  result.ell_vv = -e
  result.ell_vw = d

proc doublingStepForFlippedML(p: var Point[G2]): EllCoeffs =
  let a = (p.x * p.y).scale(TwoInv)
  let b = p.y.squared()
  let c = p.z.squared()
  let d = c + c + c
  let e = G2B * d
  let f = e + e + e
  let g = (b + f).scale(TwoInv)
  let h = (p.y + p.z).squared() - (b + c)
  let i = e - b
  let j = p.x.squared()
  let e_sq = e.squared()

  p.x = a * (b - f)
  p.y = g.squared() - (e_sq + e_sq + e_sq)
  p.z = b * h

  result.ell_0 = Twist * i
  result.ell_vw = -h
  result.ell_vv = j + j + j

proc precompute*(p: AffinePoint[G2]): G2Precomp =
  var r = p.toJacobian()
  result.coeffs = newSeqOfCap[EllCoeffs](102)
  var foundOne = false

  for i in AteLoopCount.bits():
    if not foundOne:
      foundOne = i
      continue
    result.coeffs.add(r.doublingStepForFlippedML())
    if i:
      result.coeffs.add(r.mixedAdditionStepForFlippedML(p))

  let q1 = p.mulByQ()
  let q2 = -(q1.mulByQ())

  result.coeffs.add(r.mixedAdditionStepForFlippedML(q1))
  result.coeffs.add(r.mixedAdditionStepForFlippedML(q2))
  result.q = p

proc millerLoop*(pc: G2Precomp, g1: AffinePoint[G1]): FQ12 =
  result = FQ12.one()
  var idx = 0
  var foundOne = false
  var c: EllCoeffs
  for i in AteLoopCount.bits():
    if not foundOne:
      foundOne = i
      continue
    c = pc.coeffs[idx]
    inc(idx)
    result = result.squared().mulBy024(c.ell_0, c.ell_vw.scale(g1.y),
                                       c.ell_vv.scale(g1.x))
    if i:
      c = pc.coeffs[idx]
      idx += 1
      result = result.mulBy024(c.ell_0, c.ell_vw.scale(g1.y),
                               c.ell_vv.scale(g1.x))
  c = pc.coeffs[idx]
  idx += 1
  result = result.mulBy024(c.ell_0, c.ell_vw.scale(g1.y), c.ell_vv.scale(g1.x))
  c = pc.coeffs[idx]
  result = result.mulBy024(c.ell_0, c.ell_vw.scale(g1.y), c.ell_vv.scale(g1.x))

proc pairing*(p: Point[G1], q: Point[G2]): FQ12 {.noinit, inline.} =
  result = FQ12.one()
  var optp = p.toAffine()
  var optq = q.toAffine()
  if optp.isSome() and optq.isSome():
    let pc = optq.get().precompute()
    let ores = finalExponentiation(pc.millerLoop(optp.get()))
    if ores.isSome():
      result = ores.get()

