import { ListBasicType, UintNumberType, ValueOf } from '@chainsafe/ssz';
import { BeaconApi } from '../../../../relay/implementations/beacon-api';
import { Tree } from '@chainsafe/persistent-merkle-tree';
import { bytesToHex } from '../../../../libs/typescript/ts-utils/bls';
import { sha256 } from 'ethers/lib/utils';
import {
  verifyMerkleProof,
  hashTreeRoot,
} from '../../../../libs/typescript/ts-utils/ssz-utils';
import { get } from 'node:http';

(async () => {
  const { ssz } = await import('@lodestar/types');
  const beaconApi = new BeaconApi([
    'http://unstable.prater.beacon-api.nimbus.team/',
  ]);

  const beaconStateSZZ = await fetch(
    `http://testing.mainnet.beacon-api.nimbus.team/eth/v2/debug/beacon/states/6616005`,
    {
      headers: {
        Accept: 'application/octet-stream',
      },
    },
  )
    .then(response => response.arrayBuffer())
    .then(buffer => new Uint8Array(buffer));

  const beaconState = ssz.capella.BeaconState.deserialize(beaconStateSZZ);

  console.log(
    BigInt(
      '0x' +
        bytesToHex(
          ssz.phase0.Validator.fields.exitEpoch.hashTreeRoot(
            beaconState.validators[0].exitEpoch,
          ),
        ),
    )
      .toString(2)
      .padStart(256, '0')
      .split('')
      .map(x => `"${x.toString()}"`)
      .join(','),
  );

  console.log(
    BigInt(
      '0x' +
        bytesToHex(
          ssz.phase0.Validator.hashTreeRoot(beaconState.validators[0]),
        ),
    )
      .toString(2)
      .padStart(256, '0')
      .split('')
      .map(x => `"${x.toString()}"`)
      .join(','),
  );

  // const beaconStateView = ssz.capella.BeaconState.toViewDU(beaconState);
  // const beaconStateTree = new Tree(beaconStateView.node);

  // beaconStateTree.getProof()

  // console.log(beaconState.slot);

  // console.log(ssz.capella.BeaconState.getPathInfo(['slot']).gindex);

  // console.log(
  //   beaconStateTree
  //     .getSingleProof(ssz.capella.BeaconState.getPathInfo(['slot']).gindex)
  //     .map(bytesToHex),
  // );

  // console.log(ssz.capella.BeaconState.getPathInfo(['validators']).gindex);

  // console.log(ssz.phase0.Validators.getPathInfo([0]).gindex);

  // console.log(
  //   (ssz.phase0.Validator.getPathInfo(['pubkey']).type as any).fieldsGindex
  //     .pubkey,
  // );

  // console.log(ssz.capella.BeaconState.getPathInfo(['eth1DepositIndex']).gindex);

  // console.log(beaconStateTree.getSingleProof(42n).length);

  // // const gindex = ssz.capella.BeaconState.getPathInfo(['validators', 0]).gindex;
  // const validatorsIndex = ssz.capella.BeaconState.getPathInfo([
  //   'validators',
  // ]).gindex;

  // const validatorsProof = beaconStateTree
  //   .getSingleProof(ssz.capella.BeaconState.getPathInfo(['validators']).gindex)
  //   .map(bytesToHex);

  // const validatorsView = ssz.capella.BeaconState.fields.validators.toViewDU(
  //   beaconState.validators,
  // );
  // const validatorsTree = new Tree(validatorsView.node);

  // const validatorIndex = ssz.phase0.Validators.getPathInfo([0]).gindex;

  // const validatorView =
  //   ssz.capella.BeaconState.fields.validators.elementType.toViewDU(
  //     beaconState.validators[0],
  //   );
  // const validatorTree = new Tree(validatorView.node);
  // const validatorProof = validatorsTree
  //   .getSingleProof(validatorIndex)
  //   .map(bytesToHex);

  // const pubkeyIndex =
  //   ssz.capella.BeaconState.fields.validators.elementType.getPathInfo([
  //     'withdrawableEpoch',
  //   ]);

  // console.log(pubkeyIndex);

  // const hashTreeRoot = bytesToHex(
  //   ssz.capella.BeaconState.hashTreeRoot(beaconState),
  // );

  // const pubkeyIndex = 8n;
  // const pubkeyProof = validatorTree.getSingleProof(8n).map(bytesToHex);

  // console.log('hash tree root', hashTreeRoot);
  // console.log('branch', pubkeyProof.concat(validatorProof));

  // console.log('leaf', bytesToHex(beaconState.validators[0].pubkey));
  // console.log(
  //   'pubkey hashtreeroot',
  //   bytesToHex(
  //     ssz.capella.BeaconState.fields.validators.elementType.fields.pubkey.hashTreeRoot(
  //       beaconState.validators[0].pubkey,
  //     ),
  //   ),
  // );
  // console.log(
  //   'hashed pubkey',
  //   sha256(
  //     '0x' + bytesToHex(beaconState.validators[0].pubkey).padEnd(128, '0'),
  //   ),
  // );

  // console.log(
  //   bytesToHex(ssz.phase0.Validators.hashTreeRoot(beaconState.validators)),
  // );

  // console.log(
  //   bytesToHex(ssz.phase0.Validator.hashTreeRoot(beaconState.validators[0])),
  // );

  // console.log(validatorIndex);

  // console.log(
  //   verifyMerkleProof(
  //     pubkeyProof.concat(validatorProof).concat(validatorsProof),
  //     bytesToHex(ssz.capella.BeaconState.hashTreeRoot(beaconState)),
  //     bytesToHex(
  //       ssz.capella.BeaconState.fields.validators.elementType.fields.pubkey.hashTreeRoot(
  //         beaconState.validators[0].pubkey,
  //       ),
  //     ),
  //     BigInt(
  //       '0b' +
  //         validatorsIndex.toString(2) +
  //         validatorIndex.toString(2).slice(1) +
  //         pubkeyIndex.toString(2).slice(1),
  //     ),
  //   ),
  // );

  // const pubkey = bytesToHex(beaconState.validators[0].pubkey);
  // console.log('pubkey', pubkey);

  // console.log('sha256 pubkey', sha256(beaconState.validators[0].pubkey));

  // const proof = beaconStateTree.getSingleProof(gindex).map(bytesToHex);
  // console.log('proof', proof);

  // const hashTreeRoot = bytesToHex(
  //   ssz.capella.BeaconState.fields.validators.hashTreeRoot(
  //     beaconState.validators,
  //   ),
  // );

  // console.log('validators hash tree root', hashTreeRoot);

  // const leaf = sha256('0x' + pubkey.padEnd(128, '0'));
  // console.log(leaf);

  // console.log(
  //   'valid proof',
  //   verifyMerkleProof(proof, hashTreeRoot, pubkey, gindex),
  // );

  // // ssz.capella.BeaconState.
  // const balances = ssz.capella.BeaconState.fields.balances;
  // console.log('Balances', balances.getPathInfo([5]).gindex);

  // console.log(balances.getPathInfo([5]).gindex.toString(2));

  // const balancesView = ssz.capella.BeaconState.fields.balances.toViewDU(
  //   beaconState.balances,
  // );
  // const balancesTree = new Tree(balancesView.node);

  // console.log(
  //   balancesTree
  //     .getSingleProof(balances.getPathInfo([5]).gindex)
  //     .map(bytesToHex),
  // );

  // console.log(
  //   'balances proof length',
  //   balancesTree.getSingleProof(balances.getPathInfo([5]).gindex).length,
  // );

  // console.log('State', gindex);

  // console.log(gindex.toString(2));

  // console.log(beacoStateTree.getSingleProof(gindex).map(x => bytesToHex(x)));

  // console.log(
  //   'Beacon state proof length',
  //   beacoStateTree.getSingleProof(gindex).length,
  // );

  // console.log(beaconState.balances.slice(0, 5));
})();

function calculateDaysBetweeenDates(begin: Date, end: Date): number {
  const diffTime = Math.abs(end.getTime() - begin.getTime());
  const diffDays = Math.ceil(diffTime / (1000 * 60 * 60 * 24));

  return diffDays;
}
