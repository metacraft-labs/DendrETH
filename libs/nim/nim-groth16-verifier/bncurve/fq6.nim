# Nim Barreto-Naehrig pairing-friendly elliptic curve implementation
# Copyright (c) 2018 Status Research & Development GmbH
# Licensed under either of
#  * Apache License, version 2.0, ([LICENSE-APACHE](LICENSE-APACHE))
#  * MIT license ([LICENSE-MIT](LICENSE-MIT))
# at your option.
# This file may not be copied, modified, or distributed except according to
# those terms.
import options
import fq2, fp, arith

{.deadCodeElim: on.}

const frobeniusCoeffsC1: array[4, FQ2] = [
  FQ2.one(),
  FQ2(
    c0: FQ([13075984984163199792'u64, 3782902503040509012'u64,
            8791150885551868305'u64, 1825854335138010348'u64]),
    c1: FQ([7963664994991228759'u64, 12257807996192067905'u64,
            13179524609921305146'u64, 2767831111890561987'u64])
  ),
  FQ2(
    c0: FQ([3697675806616062876'u64, 9065277094688085689'u64,
            6918009208039626314'u64, 2775033306905974752'u64]),
    c1: FQ.zero()
  ),
  FQ2(
    c0: FQ([14532872967180610477'u64, 12903226530429559474'u64,
            1868623743233345524'u64, 2316889217940299650'u64]),
    c1: FQ([12447993766991532972'u64, 4121872836076202828'u64,
            7630813605053367399'u64, 740282956577754197'u64])
  )
]

const frobeniusCoeffsC2: array[4, FQ2] = [
  FQ2.one(),
  FQ2(
    c0: FQ([8314163329781907090'u64, 11942187022798819835'u64,
            11282677263046157209'u64, 1576150870752482284'u64]),
    c1: FQ([6763840483288992073'u64, 7118829427391486816'u64,
            4016233444936635065'u64, 2630958277570195709'u64])
  ),
  FQ2(
    c0: FQ([8183898218631979349'u64, 12014359695528440611'u64,
            12263358156045030468'u64, 3187210487005268291'u64]),
    c1: FQ.zero()
  ),
  FQ2(
    c0: FQ([4938922280314430175'u64, 13823286637238282975'u64,
            15589480384090068090'u64, 481952561930628184'u64]),
    c1: FQ([3105754162722846417'u64, 11647802298615474591'u64,
            13057042392041828081'u64, 1660844386505564338'u64])
  )
]

type
  FQ6* = object
    c0*: FQ2
    c1*: FQ2
    c2*: FQ2

proc init*(c0, c1, c2: FQ2): FQ6 {.inline, noinit.} =
  result.c0 = c0
  result.c1 = c1
  result.c2 = c2

proc zero*(t: typedesc[FQ6]): FQ6 {.inline, noinit.} =
  result.c0 = FQ2.zero()
  result.c1 = FQ2.zero()
  result.c2 = FQ2.zero()

proc one*(t: typedesc[FQ6]): FQ6 {.inline, noinit.} =
  result.c0 = FQ2.one()
  result.c1 = FQ2.zero()
  result.c2 = FQ2.zero()


proc isZero*(x: FQ6): bool {.inline, noinit.} =
  result = (x.c0.isZero() and x.c1.isZero() and x.c2.isZero())

proc scale*(x: FQ6, by: FQ2): FQ6 {.inline, noinit.} =
  result.c0 = x.c0 * by
  result.c1 = x.c1 * by
  result.c2 = x.c2 * by

proc squared*(x: FQ6): FQ6 {.inline, noinit.} =
  let s0 = x.c0.squared()
  let ab = x.c0 * x.c1
  let s1 = ab + ab
  let s2 = (x.c0 - x.c1 + x.c2).squared()
  let bc = x.c1 * x.c2
  let s3 = bc + bc
  let s4 = x.c2.squared()

  result.c0 = s0 + s3.mulByNonresidue()
  result.c1 = s1 + s4.mulByNonresidue()
  result.c2 = s1 + s2 + s3 - s0 - s4

proc inverse*(x: FQ6): Option[FQ6] {.inline, noinit.} =
  let c0 = x.c0.squared() - (x.c1 * x.c2.mulByNonresidue())
  let c1 = x.c2.squared().mulByNonresidue() - (x.c0 * x.c1)
  let c2 = x.c1.squared() - (x.c0 * x.c2)
  let opt = ((x.c2 * c1 + x.c1 * c2).mulByNonresidue() +
             x.c0 * c0).inverse()
  if isSome(opt):
    let tmp = opt.get()
    result = some[FQ6](FQ6(c0: tmp * c0, c1: tmp * c1, c2: tmp * c2))
  else:
    result = none[FQ6]()

proc `+`*(x, y: FQ6): FQ6 {.noinit, inline.} =
  ## Return result of ``x + y``.
  result.c0 = x.c0 + y.c0
  result.c1 = x.c1 + y.c1
  result.c2 = x.c2 + y.c2

proc `+=`*(x: var FQ6, y: FQ6) {.noinit, inline.} =
  ## Perform inplace addition ``x = x + y``.
  x.c0 += y.c0
  x.c1 += y.c1
  x.c2 += y.c2

proc `-`*(x, y: FQ6): FQ6 {.noinit, inline.} =
  ## Return result of ``x - y``.
  result.c0 = x.c0 - y.c0
  result.c1 = x.c1 - y.c1
  result.c2 = x.c2 - y.c2

proc `-=`*(x: var FQ6, y: FQ6) {.noinit, inline.} =
  ## Perform inplace substraction ``x = x - y``.
  x.c0 -= y.c0
  x.c1 -= y.c1
  x.c2 -= y.c2

proc `*`*(x, y: FQ6): FQ6 {.noinit, inline.} =
  ## Return result of ``x * y``.
  let aa = x.c0 * y.c0
  let bb = x.c1 * y.c1
  let cc = x.c2 * y.c2
  result.c0 = ((x.c1 + x.c2) * (y.c1 + y.c2) - bb - cc).mulByNonresidue() +
              aa
  result.c1 = (x.c0 + x.c1) * (y.c0 + y.c1) - aa - bb + cc.mulByNonresidue()
  result.c2 = (x.c0 + x.c2) * (y.c0 + y.c2) - aa + bb - cc

proc `*=`*(x: var FQ6, y: FQ6) {.noinit, inline.} =
  ## Perform inplace multiplication ``x = x * y``.
  let aa = x.c0 * y.c0
  let bb = x.c1 * y.c1
  let cc = x.c2 * y.c2
  let dd = x.c1 + x.c2
  let ee = x.c0 + x.c1
  let ff = x.c0 + x.c2

  x.c0 = (dd * (y.c1 + y.c2) - bb - cc).mulByNonresidue() + aa
  x.c1 = ee * (y.c0 + y.c1) - aa - bb + cc.mulByNonresidue()
  x.c2 = ff * (y.c0 + y.c2) - aa - bb - cc

proc `-`*(x: FQ6): FQ6 {.noinit, inline.} =
  ## Negotiation of ``x``.
  result.c0 = -x.c0
  result.c1 = -x.c1
  result.c2 = -x.c2

proc frobeniusMap*(x: FQ6, power: uint64): FQ6 =
  result.c0 = x.c0.frobeniusMap(power)
  result.c1 = x.c1.frobeniusMap(power) * frobeniusCoeffsC1[power mod 6]
  result.c2 = x.c2.frobeniusMap(power) * frobeniusCoeffsC2[power mod 6]

proc `==`*(x: FQ6, y: FQ6): bool =
  ## Return ``true`` if ``a == b``.
  result = (x.c0 == y.c0) and (x.c1 == y.c1) and (x.c2 == y.c2)

proc mulByNonresidue*(x: FQ6): FQ6 =
  result.c0 = x.c2.mulByNonresidue()
  result.c1 = x.c0
  result.c2 = x.c1
