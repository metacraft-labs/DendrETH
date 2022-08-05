import { PointG1 } from "@noble/bls12-381";
import * as update from "../../../data/mainnet/updates/00290.json";
import { writeFileSync } from "fs";

let points: PointG1[] = update.next_sync_committee.pubkeys.map(x => PointG1.fromHex(x.slice(2)));

let input = {
  points: points.map(point => BigInt("0x" + point.toHex(true)).toString(2).split('')),
  aggregatedKey: BigInt(update.next_sync_committee.aggregate_pubkey).toString(2).split('')
};

writeFileSync("scripts/hash_tree_root/input.json", JSON.stringify(input));
