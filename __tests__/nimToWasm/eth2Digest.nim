import light_client_utils
import nimcrypto/hash

func eth2DigestCompare*(blockRoot: Eth2Digest): bool {.cdecl, exportc, dynlib} =
  var expected = MDigest[256].fromHex("ca6ddab42853a7aef751e6c2bf38b4ddb79a06a1f971201dcf28b0f2db2c0d61")
  expected == blockRoot
