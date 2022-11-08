# Nim Barreto-Naehrig pairing-friendly elliptic curve implementation
# Copyright (c) 2018 Status Research & Development GmbH
# Licensed under either of
#  * Apache License, version 2.0, ([LICENSE-APACHE](LICENSE-APACHE))
#  * MIT license ([LICENSE-MIT](LICENSE-MIT))
# at your option.
# This file may not be copied, modified, or distributed except according to
# those terms.
import options
import fp, arith

{.deadCodeElim: on.}

type
  FQ2* = object
    c0*: FQ
    c1*: FQ

const
  FQNonResidue = FQ([
    0x68c3488912edefaa'u64, 0x8d087f6872aabf4f'u64,
    0x51e1a24709081231'u64, 0x2259d6b14729c0fa'u64
  ])

  FQ2NonResidue* = FQ2(
    c0: FQ([
      0xf60647ce410d7ff7'u64, 0x2f3d6f4dd31bd011'u64,
      0x2943337e3940c6d1'u64, 0x1d9598e8a7e39857'u64
    ]),
    c1: FQ([
      0xd35d438dc58f0d9d'u64, 0x0a78eb28f5c70b3d'u64,
      0x666ea36f7879462c'u64, 0x0e0a77c19a07df2f'u64
    ])
  )

proc init*(c0, c1: FQ): FQ2 {.inline, noinit.} =
  result.c0 = c0
  result.c1 = c1

proc zero*(t: typedesc[FQ2]): FQ2 {.inline, noinit.} =
  result.c0 = FQ.zero()
  result.c1 = FQ.zero()

proc one*(t: typedesc[FQ2]): FQ2 {.inline, noinit.} =
  result.c0 = FQ.one()
  result.c1 = FQ.zero()

proc isZero*(x: FQ2): bool {.inline, noinit.} =
  result = (x.c0.isZero() and x.c1.isZero())

proc scale*(x: FQ2, by: FQ): FQ2 {.inline, noinit.} =
  result.c0 = x.c0 * by
  result.c1 = x.c1 * by

proc squared*(x: FQ2): FQ2 {.inline, noinit.} =
  let ab = x.c0 * x.c1
  result.c0 = (x.c1 * FQNonResidue + x.c0) * (x.c0 + x.c1) -
              ab - ab * FQNonResidue
  result.c1 = ab + ab

proc inverse*(x: FQ2): Option[FQ2] {.inline, noinit.} =
  let opt = (x.c0.squared() - (x.c1.squared() * FQNonResidue)).inverse()
  if isSome(opt):
    let tmp = opt.get()
    result = some[FQ2](FQ2(c0: x.c0 * tmp, c1: -(x.c1 * tmp)))
  else:
    result = none[FQ2]()

proc `+`*(x, y: FQ2): FQ2 {.noinit, inline.} =
  ## Return result of ``x + y``.
  result.c0 = x.c0 + y.c0
  result.c1 = x.c1 + y.c1

proc `+=`*(x: var FQ2, y: FQ2) {.noinit, inline.} =
  ## Perform inplace addition ``x = x + y``.
  x.c0 += y.c0
  x.c1 += y.c1

proc `-`*(x, y: FQ2): FQ2 {.noinit, inline.} =
  ## Return result of ``x - y``.
  result.c0 = x.c0 - y.c0
  result.c1 = x.c1 - y.c1

proc `-=`*(x: var FQ2, y: FQ2) {.noinit, inline.} =
  ## Perform inplace substraction ``x = x - y``.
  x.c0 -= y.c0
  x.c1 -= y.c1

proc `*`*(x, y: FQ2): FQ2 {.noinit, inline.} =
  ## Return result of ``x * y``.
  let aa = x.c0 * y.c0
  let bb = x.c1 * y.c1
  result.c0 = bb * FQNonResidue + aa
  result.c1 = (x.c0 + x.c1) * (y.c0 + y.c1) - aa - bb

proc `*=`*(x: var FQ2, y: FQ2) {.noinit, inline.} =
  ## Perform inplace multiplication ``x = x * y``.
  let aa = x.c0 * y.c0
  let bb = x.c1 * y.c1
  let cc = x.c1 + x.c1
  x.c0 = bb * FQNonResidue + aa
  x.c1 = cc * (y.c0 + y.c1) - aa - bb

proc `-`*(x: FQ2): FQ2 {.noinit, inline.} =
  ## Negotiation of ``x``.
  result.c0 = -x.c0
  result.c1 = -x.c1

proc frobeniusMap*(x: FQ2, power: uint64): FQ2 =
  if power mod 2 == 0:
    result = x
  else:
    result.c0 = x.c0
    result.c1 = x.c1 * FQNonResidue

proc `==`*(x: FQ2, y: FQ2): bool =
  ## Return ``true`` if ``a == b``.
  result = (x.c0 == y.c0) and (x.c1 == y.c1)

proc mulByNonresidue*(x: FQ2): FQ2 =
  result = x * FQ2NonResidue

proc fromBytes*(dst: var FQ2, src: openArray[byte]): bool {.noinit.} =
  ## Create 512bit integer FQ2 from big-endian bytes representation ``src``.
  ## Returns ``true`` if ``dst`` was successfully initialized, ``false``
  ## otherwise.
  result = false
  var value: BNU512
  if fromBytes(value, src):
    var b0: BNU256
    var b1o = value.divrem(FQ.modulus(), b0)
    if isSome(b1o):
      var c0o = FQ.init(b0)
      var c1o = FQ.init(b1o.get())
      if isSome(c0o) and isSome(c1o):
        dst = init(c0o.get(), c1o.get())
        result = true

proc fromBytes2*(dst: var FQ2, src: openArray[byte]): bool {.noinit.} =
  ## Create integer FQ2 from big-endian bytes representation ``src`` in
  ## Ethereum way.
  ## Returns ``true`` if ``dst`` was successfully initialized, ``false``
  ## otherwise.
  result = false
  if dst.c1.fromBytes2(src.toOpenArray(0, 31)) and
     dst.c0.fromBytes2(src.toOpenArray(32, 63)):
    result = true

proc toBytes*(src: FQ2,
              dst: var openArray[byte]): bool {.noinit, inline.} =
  ## Encode 512bit integer FQ2 to big-endian bytes representation ``dst``.
  ## Returns ``true`` if integer was successfully serialized, ``false``
  ## otherwise.
  var c0, c1: BNU256
  c0 = BNU256.into(src.c0)
  c1 = BNU256.into(src.c1)
  result = BNU512.into(c1, c0, FQ.modulus()).toBytes(dst)
