import { PointG1 } from "@noble/bls12-381";
import { bigint_to_array, bytesToHex } from "../../../libs/typescript/ts-utils/bls";
import { fastestTester } from "./circuit_tester";
import { expect } from "chai";
import * as update from "../../../vendor/eth2-light-client-updates/mainnet/updates/00290.json";
import { ssz } from "@chainsafe/lodestar-types";
import * as constants from "../../solidity/test/utils/constants";
import { formatJSONBlockHeader } from "../../solidity/test/utils/format";
import { sha256 } from "ethers/lib/utils";

describe("Compute signing root test", () => {
  it("Test1", async () => {
    const block_header = formatJSONBlockHeader(update.attested_header);
    const headerHash = ssz.phase0.BeaconBlockHeader.hashTreeRoot(block_header);
    const fork_data = ssz.phase0.ForkData.defaultValue();
    fork_data.currentVersion = constants.ALTAIR_FORK_VERSION;
    fork_data.genesisValidatorsRoot = constants.GENESIS_VALIDATORS_ROOT;
    const fork_data_root = ssz.phase0.ForkData.hashTreeRoot(fork_data);

    const domain = new Uint8Array(32);
    for (let i = 0; i < 4; i++) domain[i] = constants.DOMAIN_SYNC_COMMITTEE[i];
    for (let i = 0; i < 28; i++) domain[i + 4] = fork_data_root[i];

    const signing_data = ssz.phase0.SigningData.defaultValue();
    signing_data.objectRoot = headerHash;
    signing_data.domain = domain;

    const signing_root = ssz.phase0.SigningData.hashTreeRoot(signing_data);

    const circuit = await fastestTester("./scripts/compute_signing_root/compute_signing_root.circom",);

    let input = {
      headerHash: BigInt("0x" + bytesToHex(headerHash)).toString(2).padStart(256, '0').split(''),
      domain: BigInt("0x" + bytesToHex(domain)).toString(2).padStart(256, '0').split('')
    };

    const witness = await circuit.calculateWitness(input);
    let a = "";
    for (let i = 1; i <= 256; i++) {
      a += witness[i];
    }

    expect(BigInt("0x" + bytesToHex(signing_root)).toString(2).padStart(256, '0')).to.be.eq(a);
  });
});
