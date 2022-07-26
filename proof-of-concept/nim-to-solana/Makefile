solana.c : solana.nim
	cp ../nimToWasm/panicoverride.nim $(wildcard ../nimToWasm/*.h) .
	nim c -c \
	--cpu:amd64 --noMain:on --nimcache:nimcache --app:lib \
	--os:standalone --cc:clang --opt:size --listCmd --d:wasm \
	--d:noSignalHandler --d:nimPreviewFloatRoundtrip --exceptions:goto \
	--gc:none --threads:off --stackTrace:off --lineTrace:off --mm:none $<
	cp nimcache/@m*.c nimcache/stdlib_system.nim.c .
	mv @msolana.nim.c solana.c

_clean:
	rm -rf nimcache
	rm -rf panicoverride.nim $(wildcard *.h) $(wildcard *.c)

.PHONY: _clean

%:
	$(MAKE) -f solana.mk $@

clean: _clean
	$(MAKE) -f solana.mk $@

solana: solana.c
	$(MAKE) -f solana.mk $@
