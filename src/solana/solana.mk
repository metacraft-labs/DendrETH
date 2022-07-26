OUT_DIR := ../../target/program
SRC_DIR := ../../src
SOLANA_TOOLS = $(shell dirname $(shell which cargo-build-bpf))
INC_DIRS := .

C_FLAGS := -m32 -nostdinc -fno-builtin -no-builtin -fno-exceptions -fno-threadsafe-statics -fvisibility=hidden -flto -std=c99
CPP_FLAGS := $(CFLAGS)

include $(SOLANA_TOOLS)/sdk/bpf/c/bpf.mk
