/*
    __________  _____    ____              _     __
   / ____/ __ \/ ___/   / __ \____ _____  (_)___/ /
  / __/ / / / /\__ \   / /_/ / __ `/ __ \/ / __  /
 / /___/ /_/ /___/ /  / _, _/ /_/ / /_/ / / /_/ /
/_____/\____//____/  /_/ |_|\__,_/ .___/_/\__,_/
                                /_/
Groth16 EOS Verifier Copyright (c) 2022 EOS Rapid, Carter Feldman (https://eosrapid.com)

Permission is hereby granted, free of charge, to any person obtaining a copy of this software and associated documentation files (the "Software"), to deal in the Software without restriction, including without limitation the rights to use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is furnished to do so, subject to the following conditions:
The above copyright notice and this permission notice shall be included in all copies or substantial portions of the Software.
THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
*/


// Some uint256 code adapted from https://github.com/calccrypto/uint256_t (Copyright (c) 2013 - 2017 Jason Lee @ calccrypto at gmail.com, MIT License)

#pragma once
#include <eosio/eosio.hpp>

#define U256_OP_UPPER_P(x) x->elements[0]
#define U256_OP_LOWER_P(x) x->elements[1]
#define U256_OP_UPPER(x) x.elements[0]
#define U256_OP_LOWER(x) x.elements[1]

namespace rapid_uint256_basic {

typedef uint128_t u128_t;
typedef struct u256_t
{
   u128_t elements[2];
} u256_t;

static const char U256_STR_HEXDIGITS[] = "0123456789abcdef";

inline uint64_t readUint64BE(uint8_t* buffer)
{
   return (((uint64_t)buffer[0]) << 56) | (((uint64_t)buffer[1]) << 48) | (((uint64_t)buffer[2]) << 40) |
          (((uint64_t)buffer[3]) << 32) | (((uint64_t)buffer[4]) << 24) | (((uint64_t)buffer[5]) << 16) |
          (((uint64_t)buffer[6]) << 8) | (((uint64_t)buffer[7]));
}

inline void readu128BE(uint8_t* buffer, u128_t* target)
{
   *target = ((u128_t)(u128_t(readUint64BE(buffer)) << 64)) | ((u128_t)(u128_t(readUint64BE(buffer + 8))));
}

inline void readu256BE(uint8_t* buffer, u256_t* target)
{
   readu128BE(buffer, &U256_OP_UPPER_P(target));
   readu128BE(buffer + 16, &U256_OP_LOWER_P(target));
}

inline bool zero128(u128_t* number) { return (*number) == 0; }

inline bool zero256(u256_t* number) { return (zero128(&U256_OP_LOWER_P(number)) && zero128(&U256_OP_UPPER_P(number))); }

void copy128(u128_t* target, u128_t* number) { *target = *number; }

bool equal128(u128_t* number1, u128_t* number2) { return (*number1) == (*number2); }

bool gt128(u128_t* number1, u128_t* number2) { return (*number1) > (*number2); }

bool gte128(u128_t* number1, u128_t* number2) { return (*number1) >= (*number2); }

void add128(u128_t* number1, u128_t* number2, u128_t* target) { *target = (*number1) + (*number2); }

void minus128(u128_t* number1, u128_t* number2, u128_t* target) { *target = (*number1) - (*number2); }

bool equal256(u256_t* number1, u256_t* number2)
{
   return (equal128(&U256_OP_UPPER_P(number1), &U256_OP_UPPER_P(number2)) &&
           equal128(&U256_OP_LOWER_P(number1), &U256_OP_LOWER_P(number2)));
}

bool gt256(u256_t* number1, u256_t* number2)
{
   if (equal128(&U256_OP_UPPER_P(number1), &U256_OP_UPPER_P(number2))) {
      return gt128(&U256_OP_LOWER_P(number1), &U256_OP_LOWER_P(number2));
   }
   return gt128(&U256_OP_UPPER_P(number1), &U256_OP_UPPER_P(number2));
}

bool gte256(u256_t* number1, u256_t* number2) { return gt256(number1, number2) || equal256(number1, number2); }

void or128(u128_t* number1, u128_t* number2, u128_t* target) { *target = (*number1) | (*number2); }
void copy256(u256_t* target, u256_t* number)
{
   copy128(&U256_OP_UPPER_P(target), &U256_OP_UPPER_P(number));
   copy128(&U256_OP_LOWER_P(target), &U256_OP_LOWER_P(number));
}

void clear128(u128_t* target) { *target = 0; }

void clear256(u256_t* target)
{
   clear128(&U256_OP_UPPER_P(target));
   clear128(&U256_OP_LOWER_P(target));
}

void shiftl128(u128_t* number, uint32_t value, u128_t* target) { *target = (*number) << value; }
void shiftr128(u128_t* number, uint32_t value, u128_t* target) { *target = (*number) >> value; }

void mul128(u128_t* number1, u128_t* number2, u128_t* target) { *target = (*number1) * (*number2); }

void shiftl256(u256_t* number, uint32_t value, u256_t* target)
{
   if (value >= 256) {
      clear256(target);
   } else if (value == 128) {
      copy128(&U256_OP_UPPER_P(target), &U256_OP_LOWER_P(number));
      clear128(&U256_OP_LOWER_P(target));
   } else if (value == 0) {
      copy256(target, number);
   } else if (value < 128) {
      u128_t tmp1;
      u128_t tmp2;
      u256_t result;
      shiftl128(&U256_OP_UPPER_P(number), value, &tmp1);
      shiftr128(&U256_OP_LOWER_P(number), (128 - value), &tmp2);
      add128(&tmp1, &tmp2, &U256_OP_UPPER(result));
      shiftl128(&U256_OP_LOWER_P(number), value, &U256_OP_LOWER(result));
      copy256(target, &result);
   } else if ((256 > value) && (value > 128)) {
      shiftl128(&U256_OP_LOWER_P(number), (value - 128), &U256_OP_UPPER_P(target));
      clear128(&U256_OP_LOWER_P(target));
   } else {
      clear256(target);
   }
}

void shiftr256(u256_t* number, uint32_t value, u256_t* target)
{
   if (value >= 256) {
      clear256(target);
   } else if (value == 128) {
      copy128(&U256_OP_LOWER_P(target), &U256_OP_UPPER_P(number));
      clear128(&U256_OP_UPPER_P(target));
   } else if (value == 0) {
      copy256(target, number);
   } else if (value < 128) {
      u128_t tmp1;
      u128_t tmp2;
      u256_t result;
      shiftr128(&U256_OP_UPPER_P(number), value, &U256_OP_UPPER(result));
      shiftr128(&U256_OP_LOWER_P(number), value, &tmp1);
      shiftl128(&U256_OP_UPPER_P(number), (128 - value), &tmp2);
      add128(&tmp1, &tmp2, &U256_OP_LOWER(result));
      copy256(target, &result);
   } else if ((256 > value) && (value > 128)) {
      shiftr128(&U256_OP_UPPER_P(number), (value - 128), &U256_OP_LOWER_P(target));
      clear128(&U256_OP_UPPER_P(target));
   } else {
      clear256(target);
   }
}

uint32_t bits256(u256_t* number)
{
   uint32_t result = 0;
   if (!zero128(&U256_OP_UPPER_P(number))) {
      result = 128;
      u128_t up;
      copy128(&up, &U256_OP_UPPER_P(number));
      while (!zero128(&up)) {
         shiftr128(&up, 1, &up);
         result++;
      }
   } else {
      u128_t low;
      copy128(&low, &U256_OP_LOWER_P(number));
      while (!zero128(&low)) {
         shiftr128(&low, 1, &low);
         result++;
      }
   }
   return result;
}

void add256(u256_t* number1, u256_t* number2, u256_t* target)
{
   u128_t tmp;
   add128(&U256_OP_UPPER_P(number1), &U256_OP_UPPER_P(number2), &U256_OP_UPPER_P(target));
   add128(&U256_OP_LOWER_P(number1), &U256_OP_LOWER_P(number2), &tmp);
   if (gt128(&U256_OP_LOWER_P(number1), &tmp)) {
      u128_t one = 1;
      add128(&U256_OP_UPPER_P(target), &one, &U256_OP_UPPER_P(target));
   }
   add128(&U256_OP_LOWER_P(number1), &U256_OP_LOWER_P(number2), &U256_OP_LOWER_P(target));
}

void minus256(u256_t* number1, u256_t* number2, u256_t* target)
{
   u128_t tmp;
   minus128(&U256_OP_UPPER_P(number1), &U256_OP_UPPER_P(number2), &U256_OP_UPPER_P(target));
   minus128(&U256_OP_LOWER_P(number1), &U256_OP_LOWER_P(number2), &tmp);
   if (gt128(&tmp, &U256_OP_LOWER_P(number1))) {
      u128_t one = 1;
      minus128(&U256_OP_UPPER_P(target), &one, &U256_OP_UPPER_P(target));
   }
   minus128(&U256_OP_LOWER_P(number1), &U256_OP_LOWER_P(number2), &U256_OP_LOWER_P(target));
}

void or256(u256_t* number1, u256_t* number2, u256_t* target)
{
   or128(&U256_OP_UPPER_P(number1), &U256_OP_UPPER_P(number2), &U256_OP_UPPER_P(target));
   or128(&U256_OP_LOWER_P(number1), &U256_OP_LOWER_P(number2), &U256_OP_LOWER_P(target));
}
void divmod256(u256_t* l, u256_t* r, u256_t* retDiv, u256_t* retMod)
{
   u256_t copyd, adder, resDiv, resMod;
   u256_t one;
   U256_OP_UPPER(one) = 0;
   U256_OP_LOWER(one) = 1;

   uint32_t diffBits = bits256(l) - bits256(r);
   clear256(&resDiv);
   copy256(&resMod, l);
   if (gt256(r, l)) {
      copy256(retMod, l);
      clear256(retDiv);
   } else {
      shiftl256(r, diffBits, &copyd);
      shiftl256(&one, diffBits, &adder);
      if (gt256(&copyd, &resMod)) {
         shiftr256(&copyd, 1, &copyd);
         shiftr256(&adder, 1, &adder);
      }
      while (gte256(&resMod, r)) {
         if (gte256(&resMod, &copyd)) {
            minus256(&resMod, &copyd, &resMod);
            or256(&resDiv, &adder, &resDiv);
         }
         shiftr256(&copyd, 1, &copyd);
         shiftr256(&adder, 1, &adder);
      }
      copy256(retDiv, &resDiv);
      copy256(retMod, &resMod);
   }
}

static void reverseString(char* str, uint32_t length)
{
   uint32_t i, j;
   for (i = 0, j = length - 1; i < j; i++, j--) {
      uint8_t c;
      c      = str[i];
      str[i] = str[j];
      str[j] = c;
   }
}
bool tostring256(u256_t* number, uint32_t baseParam, char* out, uint32_t outLength)
{
   u256_t rDiv;
   u256_t rMod;
   u256_t base;
   copy256(&rDiv, number);
   clear256(&rMod);
   clear256(&base);
   U256_OP_LOWER(base) = baseParam;
   uint32_t offset     = 0;
   if ((baseParam < 2) || (baseParam > 16)) {
      return false;
   }
   do {
      if (offset > (outLength - 1)) {
         return false;
      }
      divmod256(&rDiv, &base, &rDiv, &rMod);
      out[offset++] = U256_STR_HEXDIGITS[(uint8_t)U256_OP_LOWER(rMod)];
   } while (!zero256(&rDiv));
   out[offset] = '\0';
   reverseString(out, offset);
   return true;
}

void negate_y_field(u256_t* field_c, u256_t* point_y, u256_t* output)
{
   // return G1Point(p.X, q - (p.Y % q));
   output->elements[0] = 0;
   output->elements[1] = 0;
   if (point_y->elements[0] != 0 || point_y->elements[1] != 0) {

      u256_t resDiv, resMod;
      resDiv.elements[0] = 0;
      resDiv.elements[1] = 0;

      resMod.elements[0] = 0;
      resMod.elements[1] = 0;
      divmod256(point_y, field_c, &resDiv, &resMod);

      minus256(field_c, &resMod, output);
   }
}
void fast_zero_buf32(char* buf32)
{
   uint64_t* b = (uint64_t*)(buf32);
   b[0]        = 0;
   b[1]        = 0;
   b[2]        = 0;
   b[3]        = 0;
}
bool fast_is_zero_buf32(char* buf32)
{
   uint64_t* b = (uint64_t*)(buf32);
   return b[0] == 0 && b[1] == 0 && b[2] == 0 && b[3] == 0;
}
void fast_write_uint64_be_buffer(uint64_t x, uint8_t* buffer)
{
   buffer[7] = (uint8_t)(x & 0xff);
   buffer[6] = (uint8_t)((x >> 8) & 0xff);
   buffer[5] = (uint8_t)((x >> 16) & 0xff);
   buffer[4] = (uint8_t)((x >> 24) & 0xff);
   buffer[3] = (uint8_t)((x >> 32) & 0xff);
   buffer[2] = (uint8_t)((x >> 40) & 0xff);
   buffer[1] = (uint8_t)((x >> 48) & 0xff);
   buffer[0] = (uint8_t)((x >> 56) & 0xff);
}
void fast_write_uint128_be_buffer(uint128_t x, uint8_t* buffer)
{
   fast_write_uint64_be_buffer((uint64_t)(x & 0xffffffffffffffff), buffer + 8);
   fast_write_uint64_be_buffer((uint64_t)(x >> 64), buffer);
}
void fast_write_u256_be_buffer(u256_t* x, uint8_t* buffer)
{
   fast_write_uint128_be_buffer(x->elements[0], buffer);
   fast_write_uint128_be_buffer(x->elements[1], buffer + 16);
}
void negate_y_field(char* point_y, char* negated_output)
{
   u256_t field_c;
   u256_t output;
   u256_t input;
   field_c.elements[0] = (((uint128_t)0x30644e72e131a029) << 64) | ((uint128_t)0xb85045b68181585d);
   field_c.elements[1] = (((uint128_t)0x97816a916871ca8d) << 64) | ((uint128_t)0x3c208c16d87cfd47);

   fast_zero_buf32(negated_output);
   if (!fast_is_zero_buf32(point_y)) {
      readu256BE((uint8_t*)point_y, &input);
      negate_y_field(&field_c, &input, &output);
      fast_write_u256_be_buffer(&output, (uint8_t*)negated_output);
   }
}
bool is_input_be_in_snark_field(uint8_t* input_big_endian)
{
   uint64_t a = readUint64BE(input_big_endian);
   if (a < 0x30644e72e131a029) {
      return true;
   } else if (a == 0x30644e72e131a029) {
      a = readUint64BE(input_big_endian + 8);
      if (a < 0xb85045b68181585d) {
         return true;
      } else if (a == 0xb85045b68181585d) {
         a = readUint64BE(input_big_endian + 16);
         if (a < 0x2833e84879b97091) {
            return true;
         } else if (a == 0x2833e84879b97091) {
            return readUint64BE(input_big_endian + 24) < 0x43e1f593f0000001;
         }
      }
   }
   return false;
}

uint8_t from_hex(char c)
{
   if (c >= '0' && c <= '9')
      return c - '0';
   if (c >= 'a' && c <= 'f')
      return c - 'a' + 10;
   if (c >= 'A' && c <= 'F')
      return c - 'A' + 10;
   eosio::check(false, "Invalid hex character");
   return 0;
}

size_t from_hex(const std::string& hex_str, char* out_data, size_t out_data_len)
{
   auto     i       = hex_str.begin();
   uint8_t* out_pos = (uint8_t*)out_data;
   uint8_t* out_end = out_pos + out_data_len;
   while (i != hex_str.end() && out_end != out_pos) {
      *out_pos = from_hex((char)(*i)) << 4;
      ++i;
      if (i != hex_str.end()) {
         *out_pos |= from_hex((char)(*i));
         ++i;
      }
      ++out_pos;
   }
   return out_pos - (uint8_t*)out_data;
}

std::string to_hex(const char* d, uint32_t s)
{
   std::string r;
   const char* to_hex = "0123456789abcdef";
   uint8_t*    c      = (uint8_t*)d;
   for (uint32_t i = 0; i < s; ++i)
      (r += to_hex[(c[i] >> 4)]) += to_hex[(c[i] & 0x0f)];
   return r;
}
} // namespace rapid_uint256_basic
