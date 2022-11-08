# Nim Barreto-Naehrig pairing-friendly elliptic curve implementation
# Copyright (c) 2018 Status Research & Development GmbH
# Licensed under either of
#  * Apache License, version 2.0, ([LICENSE-APACHE](LICENSE-APACHE))
#  * MIT license ([LICENSE-MIT](LICENSE-MIT))
# at your option.
# This file may not be copied, modified, or distributed except according to
# those terms.
import arith, options

{.deadCodeElim: on.}

template fieldImplementation(finame, fimodulus, firsquared, fircubed,
                             fionep, fiinv: untyped): untyped {.dirty.} =
  type finame* = distinct BNU256

  proc setZero*(dst: var finame) {.noinit, inline.} =
    ## Set ``zero`` representation in Fp to ``dst``.
    dst = finame([0'u64, 0'u64, 0'u64, 0'u64])

  proc isZero*(src: finame): bool {.noinit, inline.} =
    ## Check if ``src`` is ``zero``.
    result = BNU256(src).isZero()

  proc setOne*(dst: var finame) {.noinit, inline.}=
    ## Set ``one`` representation in Fp to ``dst``.
    dst = finame(fionep)

  proc zero*(t: typedesc[finame]): finame {.noinit, inline.} =
    ## Return ``zero`` representation in Fp.
    result.setZero()

  proc one*(t: typedesc[finame]): finame {.noinit, inline.} =
    ## Return ``one`` representation in Fp.
    result.setOne()

  proc modulus*(t: typedesc[finame]): BNU256 {.noinit, inline} =
    ## Return ``Fp`` modulus.
    result = fimodulus

  proc `+`*(x, y: finame): finame {.noinit, inline.} =
    ## Return result of ``x + y``.
    result = x
    add(BNU256(result), BNU256(y), fimodulus)

  proc `+=`*(x: var finame, y: finame) {.noinit, inline.} =
    ## Perform inplace addition ``x = x + y``.
    add(BNU256(x), BNU256(y), fimodulus)

  proc `-`*(x, y: finame): finame {.noinit, inline.} =
    ## Return result of ``x - y``.
    result = x
    sub(BNU256(result), BNU256(y), fimodulus)

  proc `-=`*(x: var finame, y: finame) {.noinit, inline.} =
    ## Perform inplace substraction ``x = x - y``.
    sub(BNU256(x), BNU256(y), fimodulus)

  proc `*`*(x, y: finame): finame {.noinit, inline.} =
    ## Return result of ``x * y``.
    result = x
    mul(BNU256(result), BNU256(y), fimodulus, fiinv)

  proc `*=`*(x: var finame, y: finame) {.noinit, inline.} =
    ## Perform inplace multiplication ``x = x * y``.
    mul(BNU256(x), BNU256(y), fimodulus, fiinv)

  proc `-`*(x: finame): finame {.noinit, inline.} =
    ## Negotiation of ``x``.
    result = x
    neg(BNU256(result), fimodulus)

  proc fromString*(t: typedesc[finame],
                   number: string): finame {.noinit, inline.} =
    ## Convert decimal string representation to ``Fp``.
    var numis = newSeq[finame](11)
    var acc = finame.zero()
    for number in numis.mitems():
      number = acc
      acc += finame.one()
    result.setZero()
    for ch in number:
      doAssert(ch in {'0'..'9'})
      let idx = ord(ch) - ord('0')
      result *= numis[10]
      result += numis[idx]

  proc into*(t: typedesc[BNU256], num: finame): BNU256 =
    ## Convert FR/FQ ``num`` to 256bit integer.
    result = BNU256(num)
    mul(result, BNU256.one(), BNU256(fimodulus), fiinv)

  proc init*(t: typedesc[finame], num: BNU256): Option[finame] =
    ## Initialize FR/FQ from 256bit integer ``num``.
    if num >= BNU256(fimodulus):
      result = none[finame]()
    else:
      var res: finame
      res = finame(num)
      mul(BNU256(res), BNU256(firsquared), BNU256(fimodulus), fiinv)
      result = some[finame](res)

  proc init2*(t: typedesc[finame], num: BNU256): finame =
    ## Initalize FR/FQ from 256bit integer ``num`` regardless of modulus.
    result = finame(num)
    mul(BNU256(result), BNU256(firsquared), BNU256(fimodulus), fiinv)

  proc fromBytes*(dst: var finame, src: openArray[byte]): bool {.noinit.} =
    ## Create integer FR/FQ from big-endian bytes representation ``src``.
    ## Returns ``true`` if ``dst`` was successfully initialized, ``false``
    ## otherwise.
    result = false
    var bn: BNU256
    if bn.fromBytes(src):
      var optr = finame.init(bn)
      if isSome(optr):
        dst = optr.get()
        result = true

  proc fromBytes2*(dst: var finame, src: openArray[byte]): bool {.noinit.} =
    ## Create integer FR/FQ from big-endian bytes representation ``src`` in
    ## Ethereum way (without modulo check).
    ## Returns ``true`` if ``dst`` was successfully initialized, ``false``
    ## otherwise.
    result = false
    var bn: BNU256
    if bn.fromBytes(src):
      dst = finame.init2(bn)
      result = true

  proc toBytes*(src: finame,
                dst: var openArray[byte]): bool {.noinit, inline.} =
    ## Encode integer FP/FQ to big-endian bytes representation ``dst``.
    ## Returns ``true`` if integer was successfully serialized, ``false``
    ## otherwise.
    result = BNU256.into(src).toBytes(dst)

  proc fromHexString*(dst: var finame,
                      src: string): bool {.noinit, inline.} =
    ## Create integer FP/FQ from hexadecimal string representation ``src``.
    ## Returns ``true`` if ``dst`` was successfully initialized, ``false``
    ## otherwise.
    result = false
    var bn: BNU256
    if bn.fromHexString(src):
      var optr = finame.init(bn)
      if isSome(optr):
        dst = optr.get()
        result = true

  proc inverse*(num: finame): Option[finame] =
    ## Perform inversion of ``Fp``.
    if num.isZero():
      result = none[finame]()
    else:
      var res: BNU256
      res = BNU256(num)
      invert(res, BNU256(fimodulus))
      mul(res, BNU256(fircubed), BNU256(fimodulus), fiinv)
      result = some[finame](finame(res))

  proc `==`*(a: finame, b: finame): bool {.inline, noinit.} =
    ## Return ``true`` if ``a == b``.
    result = (BNU256(a) == BNU256(b))

  proc squared*(a: finame): finame {.inline, noinit.} =
    ## Return ``a * a``.
    result = a * a

  proc pow*(a: finame, by: BNU256): finame {.inline, noinit.} =
    ## Return ``a^by``.
    result = finame.one()
    for i in by.bits():
      result = result.squared()
      if i:
        result *= a

fieldImplementation(
  FR,
  [0x43e1f593f0000001'u64, 0x2833e84879b97091'u64,
   0xb85045b68181585d'u64, 0x30644e72e131a029'u64],
  [0x1bb8e645ae216da7'u64, 0x53fe3ab1e35c59e3'u64,
   0x8c49833d53bb8085'u64, 0x0216d0b17f4e44a5'u64],
  [0x5e94d8e1b4bf0040'u64, 0x2a489cbe1cfbb6b8'u64,
   0x893cc664a19fcfed'u64, 0x0cf8594b7fcc657c'u64],
  [0xac96341c4ffffffb'u64, 0x36fc76959f60cd29'u64,
   0x666ea36f7879462e'u64, 0xe0a77c19a07df2f'u64],
  0xc2e1f593efffffff'u64
)

fieldImplementation(
  FQ,
  [0x3c208c16d87cfd47'u64, 0x97816a916871ca8d'u64,
   0xb85045b68181585d'u64, 0x30644e72e131a029'u64],
  [0xf32cfc5b538afa89'u64, 0xb5e71911d44501fb'u64,
   0x47ab1eff0a417ff6'u64, 0x06d89f71cab8351f'u64],
  [0xb1cd6dafda1530df'u64, 0x62f210e6a7283db6'u64,
   0xef7f0b0c0ada0afb'u64, 0x20fd6e902d592544'u64],
  [0xd35d438dc58f0d9d'u64, 0xa78eb28f5c70b3d'u64,
   0x666ea36f7879462c'u64, 0xe0a77c19a07df2f'u64],
  0x87d20782e4866389'u64
)
