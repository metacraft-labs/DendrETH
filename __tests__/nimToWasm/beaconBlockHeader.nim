from nimcrypto/hash import MDigest, fromHex

import ../../src/nim-light-client/light_client_utils
import ./helpers/helpers

proc beaconBlockHeaderCompare*(
    data: pointer, length: Natural
  ): bool {.cdecl, exportc, dynlib} =
  let expected = BeaconBlockHeader(
    slot: 3566048.Slot,
    proposer_index: 265275.uint64,
    parent_root: MDigest[256].fromHex("6d8394a7292d616d8825f139b09fc4dca581a9c0af44499b3283b2dfda346762"),
    state_root: MDigest[256].fromHex("2176c5be4719af3e4ac67e12c55e273468791becc5ee60b4e430a05fd289acdd"),
    body_root: MDigest[256].fromHex("916babc5bb75209f7a279ed8dd2545721ea3d6b2b6ab331c74dd4247db172b8b"))

  var beaconBlockHeader: BeaconBlockHeader
  beaconBlockHeader.deserializeSSZType(data, sizeof(BeaconBlockHeader))

  beaconBlockHeader == expected
