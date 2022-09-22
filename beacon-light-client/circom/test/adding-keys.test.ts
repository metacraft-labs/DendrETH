import { Fp, PointG1 } from "@noble/bls12-381";
import { bigint_to_array } from "../../../libs/typescript/ts-utils/bls";
import { wasm } from "./circuit_tester";
import { expect } from "chai";

describe("Add public keys test", () => {
  it("Test1", async () => {
    const a = PointG1.fromHex("a7ecfb69d8c08ee7c4155ac69adda7393593e6614b349cf83e07586a2b3fce780a54ecf31f1536b35f428a4f75263ac0");
    const b = PointG1.fromHex("a1c78c5cf35f1a5b398ce01a7b2b3333d59886e68e257d8eb06a3e893456a8b2d201a5285a0653285bdabef409a0859a");
    const add = a.add(b);
    const expectedResult = bigint_to_array(55, 7, add.toAffine()[0].value);
    expectedResult.push(...bigint_to_array(55, 7, add.toAffine()[1].value));
    const circuit = await wasm("../../vendor/circom-pairing/scripts/addfp/addfp.circom");
    const witnes = await circuit.calculateWitness({
      a: [
        bigint_to_array(55, 7, a.toAffine()[0].value),
        bigint_to_array(55, 7, a.toAffine()[1].value),
      ],
      aIsInfinity: 0,
      bIsInfinity: 0,
      b: [
        bigint_to_array(55, 7, b.toAffine()[0].value),
        bigint_to_array(55, 7, b.toAffine()[1].value),
      ]
    });

    for (let i = 0; i < expectedResult.length; i++) {
      expect(expectedResult[i]).to.be.eq(witnes[i + 1].toString());
    }
  });


  it("Addition with a zero", async () => {
    const a = PointG1.ZERO;
    const b = PointG1.fromHex("a1c78c5cf35f1a5b398ce01a7b2b3333d59886e68e257d8eb06a3e893456a8b2d201a5285a0653285bdabef409a0859a");
    const add = a.add(b);
    const expectedResult = bigint_to_array(55, 7, add.toAffine()[0].value);
    expectedResult.push(...bigint_to_array(55, 7, add.toAffine()[1].value));
    const circuit = await wasm("../../vendor/circom-pairing/scripts/addfp/addfp.circom");
    const witnes = await circuit.calculateWitness({
      a: [
        bigint_to_array(55, 7, PointG1.BASE.toAffine()[0].value),
        bigint_to_array(55, 7, PointG1.BASE.toAffine()[1].value),
      ],
      aIsInfinity: 1,
      bIsInfinity: 0,
      b: [
        bigint_to_array(55, 7, b.toAffine()[0].value),
        bigint_to_array(55, 7, b.toAffine()[1].value),
      ]
    });
    for (let i = 0; i < expectedResult.length; i++) {
      expect(expectedResult[i]).to.be.eq(witnes[i + 1].toString());
    }
  });

  it("Addition with b zero", async () => {
    const a = PointG1.fromHex("a7ecfb69d8c08ee7c4155ac69adda7393593e6614b349cf83e07586a2b3fce780a54ecf31f1536b35f428a4f75263ac0");
    const b = PointG1.ZERO;
    const add = a.add(b);
    const expectedResult = bigint_to_array(55, 7, add.toAffine()[0].value);
    expectedResult.push(...bigint_to_array(55, 7, add.toAffine()[1].value));
    const circuit = await wasm("../../vendor/circom-pairing/scripts/addfp/addfp.circom");
    const witnes = await circuit.calculateWitness({
      a: [
        bigint_to_array(55, 7, a.toAffine()[0].value),
        bigint_to_array(55, 7, a.toAffine()[1].value),
      ],
      aIsInfinity: 0,
      bIsInfinity: 1,
      b: [
        bigint_to_array(55, 7, PointG1.BASE.toAffine()[0].value),
        bigint_to_array(55, 7, PointG1.BASE.toAffine()[1].value),
      ]
    });
    for (let i = 0; i < expectedResult.length; i++) {
      expect(expectedResult[i]).to.be.eq(witnes[i + 1].toString());
    }
  });
});
