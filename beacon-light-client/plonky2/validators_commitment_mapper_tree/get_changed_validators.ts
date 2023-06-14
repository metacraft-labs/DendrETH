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

      await sleep(384000);
      continue;
    }

    console.log(beaconState.slot);

    if (prevSlot >= beaconState.slot) {
      console.log('Waiting for the next epoch.');
      await sleep(384000);
      continue;
    }

    const prevBeaconStateSSZ = new Uint8Array(
      readFileSync('prev_beacon_state.ssz'),
    );

    const prevBeaconState =
      ssz.capella.BeaconState.deserialize(prevBeaconStateSSZ);

    const validators = beaconState.validators;
    const prevValidators = prevBeaconState.validators;

    const validatorsWithIndies = validators.map((validator, index) => ({
      validator,
      index,
    }));

    const changedValidators = validatorsWithIndies.filter(
      ({ validator, index }) =>
        prevValidators[index] === undefined ||
        validator.pubkey.some(
          (byte, i) => byte !== prevValidators[index].pubkey[i],
        ) ||
        validator.withdrawalCredentials.some(
          (byte, i) => byte !== prevValidators[index].withdrawalCredentials[i],
        ) ||
        validator.effectiveBalance !== prevValidators[index].effectiveBalance ||
        validator.slashed !== prevValidators[index].slashed ||
        validator.activationEligibilityEpoch !==
          prevValidators[index].activationEligibilityEpoch ||
        validator.activationEpoch !== prevValidators[index].activationEpoch ||
        validator.exitEpoch !== prevValidators[index].exitEpoch ||
        validator.withdrawableEpoch !== prevValidators[index].withdrawableEpoch,
    );

    // TODO: push the changed validators to the tree

    console.log('#changedValidators', changedValidators.length);

    writeFileSync(
      'prev_beacon_state.ssz',
      Buffer.from(beaconStateSZZ),
      'binary',
    );

    prevSlot = beaconState.slot;

    // wait for the next epoch
    await sleep(384000);
  }
})();
