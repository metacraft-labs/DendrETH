import { writeFileSync, readFileSync, existsSync } from 'fs';
import { sleep } from '../../../libs/typescript/ts-utils/common-utils';

(async () => {
  const { ssz } = await import('@lodestar/types');

  let prevSlot = 0;

  while (true) {
    const beaconStateSZZ = await fetch(
      `https://floral-chaotic-sea.discover.quiknode.pro/c1133f4fcc19bbe54fa6e4caf0c05ac79ec7d300/eth/v2/debug/beacon/states/head`,
      {
        headers: {
          Accept: 'application/octet-stream',
        },
      },
    )
      .then(response => response.arrayBuffer())
      .then(buffer => new Uint8Array(buffer));

    const beaconState = ssz.capella.BeaconState.deserialize(beaconStateSZZ);

    if (!existsSync('prev_beacon_state.ssz')) {
      console.log('prev_beacon_state.ssz does not exist. Creating it.');

      prevSlot = beaconState.slot;

      writeFileSync(
        'prev_beacon_state.ssz',
        Buffer.from(beaconStateSZZ),
        'binary',
      );

      await sleep(12000);
      continue;
    }

    console.log(beaconState.slot);

    if (prevSlot >= beaconState.slot) {
      console.log('Waiting for the next epoch.');
      await sleep(12000);
      continue;
    }

    const prevBeaconStateSSZ = new Uint8Array(
      readFileSync('prev_beacon_state.ssz'),
    );

    const prevBeaconState =
      ssz.capella.BeaconState.deserialize(prevBeaconStateSSZ);

    const balances = beaconState.balances;
    const prevBalances = prevBeaconState.balances;

    const balancesWithIndices = balances.map((balance, index) => ({
      balance: balance,
      index,
    }));

    const changedBalances = balancesWithIndices.filter(
      ({ balance, index }) =>
        prevBalances[index] === undefined || balance !== prevBalances[index],
    );

    // TODO: push the changed validators to the tree

    console.log('#changedBalances', changedBalances.length);

    console.log(
      'changed indices',
      changedBalances.map(({ index }) => index),
    );

    writeFileSync(
      'prev_beacon_state.ssz',
      Buffer.from(beaconStateSZZ),
      'binary',
    );

    prevSlot = beaconState.slot;

    // wait for the next slot
    await sleep(12000);
  }
})();
