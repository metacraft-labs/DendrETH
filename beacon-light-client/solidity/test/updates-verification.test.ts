import { expect } from "chai";

import { init, PublicKey, Signature } from "@chainsafe/bls";
import { ssz } from "@chainsafe/lodestar-types";

import { formatJSONBlockHeader } from "./utils/format";
import { getMessage, getAggregatePubkey } from "./utils";
import * as constants from "./utils/constants";

import update1 from "../../../vendor/eth2-light-client-updates/mainnet/updates/00290.json";
import update2 from "../../../vendor/eth2-light-client-updates/mainnet/updates/00291.json";

describe("Verification", async () => {
  it("Sync committee signature verification", async () => {
    // Extract the signed message
    const block_header = formatJSONBlockHeader(update2.attested_header);
    const hash = ssz.phase0.BeaconBlockHeader.hashTreeRoot(block_header);
    const message: Uint8Array = getMessage(hash, constants.ALTAIR_FORK_VERSION);

    // Calculate aggregate validators public key
    const aggregatePubkey = getAggregatePubkey(update1, update2);
    const signature = update2.sync_aggregate.sync_committee_signature;

    // Verify the message was signed with this aggregate validators public key
    await init("herumi");
    const p = PublicKey.fromHex(aggregatePubkey);
    const s = Signature.fromHex(signature);

    expect(s.verify(p, message)).to.be.true;
  });
});
