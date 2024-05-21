import { getBeaconApi } from "@dendreth/relay/implementations/beacon-api";
import common_config from '../common_config.json';
import { Tree } from "@chainsafe/persistent-merkle-tree/lib/tree";
import { bytesToHex, hexToBytes } from "@dendreth/utils/ts-utils/bls";
import { sha256 } from "ethers/lib/utils";

(async function() {
    const { ssz } = await import("@lodestar/types");
    const api = await getBeaconApi(common_config["beacon-node"]);

    const { beaconState } = (await api.getBeaconState(9111936n))!;
    console.log('pubkey', bytesToHex(beaconState.validators[0].pubkey));

    beaconState.validators = beaconState.validators.slice(0, 19);

    const validatorsViewDU =
        ssz.deneb.BeaconState.fields.validators.toViewDU(
            beaconState.validators,
        );

    const newValidatorsTree = new Tree(validatorsViewDU.node.left);
    console.log('validators root', bytesToHex(newValidatorsTree.getNode(1n).root));

    const validator = beaconState.validators[0];
    const hashTreeRoot = bytesToHex(ssz.phase0.Validator.hashTreeRoot(validator));
    console.log('hash_tree_root', hashTreeRoot);

    const validator2 = beaconState.validators[1];
    const hashTreeRoot2 = bytesToHex(ssz.phase0.Validator.hashTreeRoot(validator));
    const inner = sha256(hexToBytes(hashTreeRoot + hashTreeRoot2));
    console.log('inner:', inner);
})();
