import { c } from "circom_tester";
import { expect } from "chai";
import { bigint_to_array, hexToBytes, utils } from '../../../libs/typescript/ts-utils/bls';

describe("Hash to field message test", () => {
  it("Test 1", async () => {
    const circuit = await c("./scripts/hash_to_field/hash_to_field.circom");

    const witnes = await circuit.calculateWitness({
      in: [0, 1, 0, 1, 0, 0, 0, 0, 0, 1, 0, 1, 1, 1, 1, 0, 1, 0, 0, 0, 0, 1, 1, 1, 0, 0, 1, 1, 0, 1, 0, 1, 1, 0, 0, 0, 0, 1, 1, 0, 1, 0, 1, 1, 1, 1, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 1, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 0, 1, 1, 0, 0, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 1, 1, 0, 0, 1, 0, 0, 0, 1, 1, 1, 1, 0, 1, 1, 0, 1, 1, 0, 0, 0, 0, 1, 0, 1, 1, 1, 0, 0, 1, 0, 0, 0, 0, 0, 1, 1, 0, 1, 1, 0, 0, 1, 1, 0, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 1, 1, 1, 0, 0, 0, 0, 1, 1, 0, 1, 1, 1, 0, 0, 1, 0, 1, 0, 0, 1, 0, 0, 1, 0, 1, 0, 0, 1, 1, 0, 1, 0, 1, 1, 1, 0, 1, 1, 0, 1, 1, 0, 0, 1, 1, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 0, 1, 0, 0, 1, 1, 1, 1, 1, 1, 0, 1, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 1, 1, 0, 1, 1, 0, 1, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 0, 1, 0, 0, 1]
    });

    const u = await utils.hashToField(hexToBytes("0x505e873586be492495799d1e47b61720d9a0a70dca4a6bb661127e9207687049"), 2);
    const hash = [
      [
        bigint_to_array(55, 7, u[0][0]),
        bigint_to_array(55, 7, u[0][1])
      ],
      [
        bigint_to_array(55, 7, u[1][0]),
        bigint_to_array(55, 7, u[1][1])
      ]
    ];

    for (var i = 0; i < 2; i++) {
      for (var j = 0; j < 2; j++) {
        for (var k = 0; k < 7; k++) {
          expect(witnes[i * 14 + j * 7 + k + 1].toString()).to.be.eq(hash[i][j][k]);
        }
      }
    }
  });
});
