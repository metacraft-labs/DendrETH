import { Tree } from "@chainsafe/persistent-merkle-tree/lib";
import { getBeaconApi } from "@dendreth/relay/implementations/beacon-api";
import { bytesToHex } from "@dendreth/utils/ts-utils/bls";
import { hexToBits } from "@dendreth/utils/ts-utils/hex-utils";



(async () => {
    const beaconApi = await getBeaconApi(['http://testing.mainnet.beacon-api.nimbus.team/']);

    const slot = await beaconApi.getHeadSlot();
    const currentSSZFork = await beaconApi.getCurrentSSZ(slot);
    const { beaconState } =
        (await beaconApi.getBeaconState(slot))!;
    const balancesView = currentSSZFork.BeaconState.fields.balances.toViewDU(
        beaconState.balances,
    );

    const balancesTree = new Tree(balancesView.node);

    const balanceZeroIndex =
        currentSSZFork.BeaconState.fields.balances.getPathInfo([0]).gindex;

    const balances: number[][] = [];

    balances.push(
        hexToBits(
            bytesToHex(balancesTree.getNode(balanceZeroIndex + BigInt(0)).root),
        ),
    );

    console.log(JSON.stringify(balances));

    console.log(beaconState.balances[0]);
    console.log(beaconState.balances[1]);
    console.log(beaconState.balances[2]);
    console.log(beaconState.balances[3]);
})();