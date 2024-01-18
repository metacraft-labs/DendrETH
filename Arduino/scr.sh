#!/usr/bin/env bash

rm -rf verify/mycache

ROOT="$(git rev-parse --show-toplevel)"

cp "${ROOT}/vendor/nim/lib/nimbase.h" "Arduino/verify"

nim c -r \
  --cpu:esp \
  -d:release \
  -d:danger \
  --opt:size \
  --verbosity:3 \
  --os:standalone \
  --noMain:on \
  --deadCodeElim:on \
  --d:CTT_32=1 \
  --d:CTT_ASM=false \
  --nimcache:Arduino/verify/mycache \
  --lib:"${ROOT}/vendor/nim/lib/" "${ROOT}/contracts/cosmos/verifier/verifier-constantine/lib/nim/verify/verify.nim"

rm "${ROOT}/verify/@"
rm Arduino/verify/@

cp -r "${ROOT}/Arduino/verify/mycache/." "${ROOT}/Arduino/verify"
