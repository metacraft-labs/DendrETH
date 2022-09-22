import { Fp, PointG1 } from "@noble/bls12-381";
import { bigint_to_array } from "../../../libs/typescript/ts-utils/bls";
import { wasm } from "circom_tester";
import { expect } from "chai";

describe("Is supermajority test", () => {
  it("When there is no supermajority", async () => {
    const circuit = await wasm("./scripts/is_supermajority/is_supermajority.circom");
    let bitmask: number[] = [];

    for (let i = 0; i < 512; i++) {
      bitmask.push(i < 341 ? 1 : 0);
    }

    const witnes = await circuit.calculateWitness({
      bitmask: bitmask
    });

    expect(witnes[1]).to.be.eq(0n);
  });


  it("When there is a supermajority", async () => {
    const circuit = await wasm("./scripts/is_supermajority/is_supermajority.circom");
    let bitmask: number[] = [];

    for (let i = 0; i < 512; i++) {
      bitmask.push(i <= 341 ? 1 : 0);
    }

    const witnes = await circuit.calculateWitness({
      bitmask: bitmask
    });

    expect(witnes[1]).to.be.eq(1n);
  });
});
