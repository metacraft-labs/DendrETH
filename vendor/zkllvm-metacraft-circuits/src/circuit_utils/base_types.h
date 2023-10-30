#include <stdint.h>

#pragma once

using Byte = unsigned char;
using Bytes32 = std::array<Byte, 32>;

#ifdef __ZKLLVM__
#define assert_true(c) \
    { __builtin_assigner_exit_check(c); }
#else
#define assert_true(c) \
    { assert(c); }
#endif