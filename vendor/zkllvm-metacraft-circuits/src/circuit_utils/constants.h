#pragma once

#include <stdint.h>

constexpr uint64_t SLOTS_PER_EPOCH = 32;
constexpr uint64_t SLOTS_PER_HISTORICAL_ROOT = 8192;

constexpr uint64_t BEACON_STATE_SLOT_GINDEX = 34;
constexpr uint64_t BEACON_STATE_PREVIOUS_JUSTIFIED_CHECKPOINT_GINDEX = 50;
constexpr uint64_t BEACON_STATE_CURRENT_JUSTIFIED_CHECKPOINT_GINDEX = 51;
constexpr uint64_t BEACON_STATE_JUSTIFICATION_BITS_GINDEX = 49;
constexpr uint64_t BEACON_STATE_FINALIZED_CHECKPOINT_GINDEX = 52;
constexpr uint64_t BEACON_STATE_BLOCK_ROOTS_GINDEX = 37;
constexpr uint64_t DEPTH18_START_BLOCK_ROOTS_GINDEX = 303104;