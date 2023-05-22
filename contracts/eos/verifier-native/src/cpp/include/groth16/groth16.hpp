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

#include <eosio/eosio.hpp>
#include <eosio/crypto_ext.hpp>
#include "rapiduint256.hpp"

namespace groth16 {

const int NUMBER_OF_INPUTS = 2;


const uint8_t g16_alpha1[] = {45,77,154,167,227,2,217,223,65,116,157,85,7,148,157,5,219,234,51,251,177,108,100,59,34,245,153,162,190,109,242,226,20,190,221,80,60,55,206,176,97,216,236,96,32,159,227,69,206,137,131,10,25,35,3,1,240,118,202,255,0,77,25,38};
const uint8_t g16_beta2[] ={9,103,3,47,203,247,118,209,175,201,133,248,136,119,241,130,211,132,128,166,83,242,222,202,169,121,76,188,59,243,6,12,14,24,120,71,173,76,121,131,116,208,214,115,43,245,1,132,125,214,139,192,224,113,36,30,2,19,188,127,193,61,183,171,48,76,251,209,224,138,112,74,153,245,232,71,217,63,140,60,170,253,222,196,107,122,13,55,157,166,154,77,17,35,70,167,23,57,193,177,164,87,168,199,49,49,35,210,77,47,145,146,248,150,183,198,62,234,5,169,213,127,6,84,122,208,206,200};
const uint8_t g16_gamma2[] = {25,142,147,147,146,13,72,58,114,96,191,183,49,251,93,37,241,170,73,51,53,169,231,18,151,228,133,183,174,243,18,194,24,0,222,239,18,31,30,118,66,106,0,102,94,92,68,121,103,67,34,212,247,94,218,221,70,222,189,92,217,146,246,237,9,6,137,208,88,95,240,117,236,158,153,173,105,12,51,149,188,75,49,51,112,179,142,243,85,172,218,220,209,34,151,91,18,200,94,165,219,140,109,235,74,171,113,128,141,203,64,143,227,209,231,105,12,67,211,123,76,230,204,1,102,250,125,170};
const uint8_t g16_delta2[] =  {25,142,147,147,146,13,72,58,114,96,191,183,49,251,93,37,241,170,73,51,53,169,231,18,151,228,133,183,174,243,18,194,24,0,222,239,18,31,30,118,66,106,0,102,94,92,68,121,103,67,34,212,247,94,218,221,70,222,189,92,217,146,246,237,9,6,137,208,88,95,240,117,236,158,153,173,105,12,51,149,188,75,49,51,112,179,142,243,85,172,218,220,209,34,151,91,18,200,94,165,219,140,109,235,74,171,113,128,141,203,64,143,227,209,231,105,12,67,211,123,76,230,204,1,102,250,125,170};

const uint8_t g16_snark_ic[] =  {44,58,92,241,178,191,106,9,43,111,250,195,89,158,220,160,198,118,17,88,54,145,38,228,148,106,158,227,152,229,108,244,21,187,108,146,122,68,99,248,168,45,191,170,162,73,139,35,143,209,160,254,247,67,172,242,198,104,191,151,79,105,218,145,27,176,179,247,123,119,189,77,235,198,125,190,25,217,25,133,107,27,213,158,71,101,189,215,86,4,147,75,224,51,188,57,1,6,187,242,243,170,93,84,117,39,248,108,217,238,176,16,89,29,251,161,156,150,36,49,107,138,210,255,227,112,78,234,43,115,238,165,20,87,175,219,94,65,87,167,79,201,210,162,125,171,4,62,76,158,54,32,126,206,43,221,136,248,120,230,19,225,31,211,76,37,112,116,78,250,32,117,57,137,77,5,45,76,184,110,97,194,202,75,140,210,75,66,22,83,36,85};

void calculate_vk_x(char** inputs, int input_count, char* output)
{
   char tmp_b[64] = {0};
   char vk_x[128] = {0};
   int  i         = 0;
   eosio::check(NUMBER_OF_INPUTS == input_count, "invalid input size");
   for (i = 0; i < NUMBER_OF_INPUTS; i++) {
      eosio::check(rapid_uint256_basic::is_input_be_in_snark_field((uint8_t*)(inputs[i])), "input must be in field");
      eosio::check(eosio::alt_bn128_mul((const char*)(&g16_snark_ic[(i + 1) * 64]), 64, (const char*)(inputs[i]), 32,
                                        &tmp_b[0], 64) == 0,
                   "error multiplying snark ic by input");
      if ((i & 1) == 1) {
         eosio::check(eosio::alt_bn128_add(&vk_x[0], 64, &tmp_b[0], 64, &vk_x[64], 64) == 0,
                      "error updating vk_x calculation");
      } else {
         eosio::check(eosio::alt_bn128_add(&vk_x[64], 64, &tmp_b[0], 64, &vk_x[0], 64) == 0,
                      "error updating vk_x calculation");
      }
   }
   if ((i & 1) == 1) {
      eosio::check(eosio::alt_bn128_add((const char*)(&g16_snark_ic[0]), 64, &vk_x[0], 64, output, 64) == 0,
                   "error performing final vk_x update");
   } else {
      eosio::check(eosio::alt_bn128_add((const char*)(&g16_snark_ic[0]), 64, &vk_x[64], 64, output, 64) == 0,
                   "error performing final vk_x update");
   }
}
void calculate_vk_x(std::vector<std::string> input, char* output)
{
   char tmp[32]   = {0};
   char tmp_b[64] = {0};
   char vk_x[128] = {0};
   eosio::check(NUMBER_OF_INPUTS == input.size(), "invalid input size");
   int i = 0;
   for (i = 0; i < NUMBER_OF_INPUTS; i++) {
      rapid_uint256_basic::from_hex(input.at(i), (char*)(&tmp[0]), 32);
      eosio::check(rapid_uint256_basic::is_input_be_in_snark_field((uint8_t*)tmp), "input must be in field");

      eosio::check(eosio::alt_bn128_mul((const char*)(&g16_snark_ic[(i + 1) * 64]), 64, (const char*)(&tmp[0]), 32,
                                        &tmp_b[0], 64) == 0,
                   "error multiplying snark ic by input");

      if ((i & 1) == 1) {
         eosio::check(eosio::alt_bn128_add(&vk_x[0], 64, &tmp_b[0], 64, &vk_x[64], 64) == 0,
                      "error updating vk_x calculation");
      } else {
         eosio::check(eosio::alt_bn128_add(&vk_x[64], 64, &tmp_b[0], 64, &vk_x[0], 64) == 0,
                      "error updating vk_x calculation");
      }
   }
   if ((i & 1) == 1) {
      eosio::check(eosio::alt_bn128_add((const char*)(&g16_snark_ic[0]), 64, &vk_x[0], 64, output, 64) == 0,
                   "error performing final vk_x update");
   } else {
      eosio::check(eosio::alt_bn128_add((const char*)(&g16_snark_ic[0]), 64, &vk_x[64], 64, output, 64) == 0,
                   "error performing final vk_x update");
   }
}
int verify_groth16_proof(std::vector<std::string> input,
                         std::vector<std::string> proof_a,
                         std::vector<std::string> proof_b,
                         std::vector<std::string> proof_c)
{
   char pairing_buffer[768] = {0};
   char d_proof_a_y[32]     = {0};
   char vk_x[64]            = {0};
   calculate_vk_x(input, &vk_x[0]);

   rapid_uint256_basic::from_hex(proof_a.at(0), (char*)(&pairing_buffer[0]), 32);
   rapid_uint256_basic::from_hex(proof_a.at(1), (char*)(&d_proof_a_y[0]), 32);

   rapid_uint256_basic::negate_y_field(&d_proof_a_y[0], &pairing_buffer[32]);

   rapid_uint256_basic::from_hex(proof_b.at(0), (char*)(&pairing_buffer[64]), 32);
   rapid_uint256_basic::from_hex(proof_b.at(1), (char*)(&pairing_buffer[64 + 32]), 32);
   rapid_uint256_basic::from_hex(proof_b.at(2), (char*)(&pairing_buffer[64 + 64]), 32);
   rapid_uint256_basic::from_hex(proof_b.at(3), (char*)(&pairing_buffer[64 + 96]), 32);

   memcpy(&pairing_buffer[192], &g16_alpha1[0], 64);
   memcpy(&pairing_buffer[192 + 64], &g16_beta2[0], 128);

   memcpy(&pairing_buffer[192 * 2], &vk_x[0], 64);

   memcpy(&pairing_buffer[192 * 2 + 64], &g16_gamma2[0], 128);

   rapid_uint256_basic::from_hex(proof_c.at(0), (char*)(&pairing_buffer[192 * 3]), 32);
   rapid_uint256_basic::from_hex(proof_c.at(1), (char*)(&pairing_buffer[192 * 3 + 32]), 32);
   memcpy(&pairing_buffer[192 * 3 + 64], &g16_delta2[0], 128);

   auto ret = eosio::alt_bn128_pair(&pairing_buffer[0], 768);
   eosio::check(ret != -1, "alt_bn128_pair error");
   return ret == 0 ? 1 : 0;
}
} // namespace demoverifier
