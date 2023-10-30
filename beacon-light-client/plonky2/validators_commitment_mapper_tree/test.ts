import { sha256 } from 'ethers/lib/utils';
import { bytesToHex, formatHex } from '@dendreth/utils/ts-utils/bls';
import { hashTreeRoot } from '@dendreth/utils/ts-utils/ssz-utils';
import { readFileSync } from 'fs';

function bitArrayToByteArray(hash: number[]): Uint8Array {
  const result = new Uint8Array(32);

  for (let byte = 0; byte < 32; ++byte) {
    let value = 0;
    for (let bit = 0; bit < 8; ++bit) {
      value += 2 ** (7 - bit) * hash[byte * 8 + bit];
    }
    result[byte] = value;
  }
  return result;
}

export function calculateValidatorsAccumulator(
  validatorsPubkeys: Uint8Array[],
  eth1DepositIndexes: Uint8Array[],
): string {
  const leaves = validatorsPubkeys.map((pubkey, i) => {
    const validatorPubkey = bytesToHex(pubkey).padStart(96, '0');
    const eth1DepositIndex = bytesToHex(eth1DepositIndexes[i]).padStart(
      16,
      '0',
    );

    return sha256(
      '0x' + formatHex(validatorPubkey) + formatHex(eth1DepositIndex),
    );
  });

  return hashTreeRoot(leaves, 32);
}

(async () => {
  let elements = JSON.parse(readFileSync('output1.json', 'utf-8')).slice(0, 4);
  let pubkeys: any[] = [];
  let eth1DepositIndexes: any[] = [];
  for (const element of elements) {
    pubkeys.push(
      new Uint8Array(element.validator_pubkey.split(',').map(Number)),
    );
    eth1DepositIndexes.push(
      new Uint8Array(
        element.validator_eth1_deposit_index.split(',').map(Number),
      ),
    );
  }

  // console.log(calculateValidatorsAccumulator(pubkeys, eth1DepositIndexes));

  console.log(
    '0x' +
      bytesToHex(
        bitArrayToByteArray([
          0, 1, 1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 1,
          1, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 1, 1, 0, 1, 0, 1, 0, 0, 0, 0, 0, 0,
          1, 1, 0, 1, 0, 1, 1, 1, 1, 1, 0, 1, 0, 1, 0, 1, 1, 1, 1, 0, 0, 1, 0,
          0, 1, 1, 1, 1, 0, 1, 0, 0, 1, 1, 1, 1, 1, 0, 1, 0, 0, 1, 0, 1, 0, 1,
          0, 1, 1, 1, 1, 0, 0, 0, 1, 0, 0, 0, 1, 1, 1, 1, 0, 0, 1, 1, 1, 1, 1,
          1, 0, 1, 1, 0, 1, 1, 1, 0, 1, 0, 0, 1, 0, 0, 1, 1, 1, 1, 0, 0, 0, 1,
          0, 1, 0, 0, 0, 1, 0, 0, 1, 1, 1, 0, 0, 1, 0, 1, 0, 0, 1, 1, 0, 0, 0,
          0, 0, 1, 0, 0, 1, 0, 1, 0, 0, 0, 0, 1, 1, 0, 1, 1, 1, 1, 0, 0, 0, 1,
          0, 1, 1, 1, 0, 0, 1, 1, 0, 0, 1, 0, 1, 0, 1, 0, 1, 1, 0, 1, 1, 0, 0,
          0, 0, 1, 0, 0, 1, 1, 1, 1, 1, 1, 0, 1, 1, 0, 1, 0, 0, 1, 0, 0, 0, 0,
          0, 0, 1, 1, 0, 0, 0, 1, 1, 0, 0, 1, 1, 1, 0, 1, 0, 0, 0, 0, 0, 1, 0,
          0, 0, 0,
        ]),
      ),
  );

  // console.log(
  //   calculateValidatorsAccumulator(
  //     [
  //       new Uint8Array([
  //         137, 188, 242, 44, 145, 165, 96, 217, 93, 9, 193, 25, 38, 100, 238,
  //         161, 186, 171, 7, 128, 182, 212, 68, 28, 163, 157, 28, 181, 9, 75, 23,
  //         123, 23, 244, 122, 103, 177, 111, 185, 114, 191, 211, 183, 139, 96,
  //         47, 254, 238,
  //       ]),
  //     ],
  //     [new Uint8Array([11, 174, 12, 0, 0, 0, 0, 0])],
  //   ),
  // );

  // console.log(
  //   bytesToHex(
  //     bitArrayToByteArray([
  //       0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 1, 0, 0, 1, 1, 1, 0, 1, 0, 1, 0,
  //       0, 0, 0, 1, 0, 1, 1, 0, 1, 0, 1, 0, 0, 1, 1, 0, 0, 0, 1, 1, 1, 1, 1, 1,
  //       1, 1, 1, 0, 1, 0, 1, 0, 1, 0, 1, 1, 1, 1, 1, 0, 0, 1, 1, 1, 0, 0, 0, 0,
  //       1, 1, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 1, 1, 0, 1, 1, 0, 1, 0, 1,
  //       1, 0, 0, 1, 0, 1, 0, 1, 0, 1, 1, 0, 1, 0, 0, 0, 1, 1, 1, 0, 0, 1, 1, 0,
  //       0, 1, 0, 1, 0, 0, 0, 0, 1, 1, 0, 0, 0, 1, 0, 1, 0, 1, 1, 1, 0, 1, 1, 0,
  //       0, 1, 1, 0, 1, 1, 0, 1, 0, 1, 0, 1, 1, 1, 1, 1, 0, 1, 0, 0, 0, 1, 1, 1,
  //       1, 0, 0, 0, 0, 1, 1, 1, 1, 1, 0, 1, 0, 1, 1, 0, 0, 0, 0, 1, 1, 0, 1, 0,
  //       0, 0, 0, 0, 0, 1, 0, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 0, 1, 0, 1, 1, 0,
  //       1, 0, 0, 0, 1, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 0,
  //       1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1,
  //     ]),
  //   ),
  // );

  // console.log(
  //   calculateValidatorsAccumulator(
  //     [
  //       new Uint8Array([
  //         149, 120, 130, 150, 31, 83, 37, 15, 155, 43, 12, 161, 173, 91, 95, 79,
  //         193, 168, 156, 58, 85, 205, 45, 187, 163, 223, 158, 133, 31, 6, 201,
  //         62, 159, 226, 230, 145, 151, 24, 132, 162, 105, 212, 228, 15, 61, 5,
  //         70, 4,
  //       ]),
  //     ],
  //     [new Uint8Array([12, 174, 12, 0, 0, 0, 0, 0])],
  //   ),
  // );

  // console.log('------ level2 -------');

  // console.log(
  //   bytesToHex(
  //     bitArrayToByteArray([
  //       1, 0, 1, 0, 0, 1, 0, 0, 1, 0, 1, 0, 0, 1, 0, 0, 1, 1, 1, 1, 1, 1, 0, 0,
  //       0, 0, 1, 0, 0, 0, 1, 0, 1, 1, 1, 1, 1, 0, 1, 0, 1, 1, 0, 0, 1, 0, 0, 0,
  //       0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 1, 1, 1, 0, 0,
  //       0, 1, 1, 0, 0, 1, 0, 0, 0, 1, 1, 0, 1, 1, 1, 0, 1, 1, 0, 0, 1, 0, 1, 1,
  //       0, 1, 0, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 0, 0, 1, 0, 1, 1, 1, 1, 1,
  //       1, 1, 1, 0, 1, 0, 1, 0, 0, 1, 1, 1, 0, 0, 1, 0, 0, 0, 1, 0, 0, 1, 1, 0,
  //       0, 1, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 1, 0, 1, 1, 1, 1,
  //       1, 0, 0, 1, 1, 0, 0, 1, 0, 0, 1, 1, 1, 0, 0, 1, 1, 1, 0, 0, 0, 0, 0, 0,
  //       1, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 1, 0, 1, 0, 0, 0, 1,
  //       1, 1, 1, 1, 0, 0, 0, 1, 0, 0, 0, 0, 1, 1, 1, 0, 0, 0, 0, 1, 0, 0, 1, 0,
  //       1, 0, 0, 0, 0, 1, 1, 0, 1, 0, 1, 0, 0, 1, 1, 1,
  //     ]),
  //   ),
  // );

  // console.log(
  //   calculateValidatorsAccumulator(
  //     [
  //       new Uint8Array([
  //         137, 188, 242, 44, 145, 165, 96, 217, 93, 9, 193, 25, 38, 100, 238,
  //         161, 186, 171, 7, 128, 182, 212, 68, 28, 163, 157, 28, 181, 9, 75, 23,
  //         123, 23, 244, 122, 103, 177, 111, 185, 114, 191, 211, 183, 139, 96,
  //         47, 254, 238,
  //       ]),
  //       new Uint8Array([
  //         149, 120, 130, 150, 31, 83, 37, 15, 155, 43, 12, 161, 173, 91, 95, 79,
  //         193, 168, 156, 58, 85, 205, 45, 187, 163, 223, 158, 133, 31, 6, 201,
  //         62, 159, 226, 230, 145, 151, 24, 132, 162, 105, 212, 228, 15, 61, 5,
  //         70, 4,
  //       ]),
  //     ],
  //     [
  //       new Uint8Array([11, 174, 12, 0, 0, 0, 0, 0]),
  //       new Uint8Array([12, 174, 12, 0, 0, 0, 0, 0]),
  //     ],
  //   ),
  // );

  // const beaconApi = new BeaconApi([
  //   'http://unstable.mainnet.beacon-api.nimbus.team',
  // ]);

  // const { beaconState } = await beaconApi.getBeaconState(6953401);

  // // const hasherResult = bytesToHex(
  // //   ssz.phase0.Validators.hashTreeRoot(beaconState.validators.slice(0, 32)),
  // // );

  // // console.log(hasherResult);

  // const validators = beaconState.validators
  //   .slice(0, 32)
  //   .map(validator => ssz.phase0.Validator.hashTreeRoot(validator));

  // const num = bytesToHex(ssz.UintNum64.hashTreeRoot(32));

  // const balancesView = ssz.capella.BeaconState.fields.balances.toViewDU(
  //   beaconState.balances,
  // );

  // const balancesTree = new Tree(balancesView.node);

  // const balanceZeroIndex = ssz.capella.BeaconState.fields.balances.getPathInfo([
  //   0,
  // ]).gindex;

  // const balances: Uint8Array[] = [];

  // for (let i = 0; i < 8; i++) {
  //   balances.push(balancesTree.getNode(balanceZeroIndex + BigInt(i)).root);
  // }

  // ssz.capella.BeaconState;
  // const result = ssz.capella.BeaconState.fields.balances.hashTreeRoot(
  //   beaconState.balances.slice(0, 32),
  // );

  // beaconState.balances = beaconState.balances.slice(0, 32);
  // beaconState.validators = beaconState.validators.slice(0, 32);

  // console.log(
  //   hexToBits(
  //     bytesToHex(ssz.capella.BeaconState.hashTreeRoot(beaconState)),
  //   ).join(','),
  // );

  // console.log(
  //   '----------------------------------------------------------------',
  // );

  // const beaconStateView = ssz.capella.BeaconState.toViewDU(beaconState);
  // const beaconStateTree = new Tree(beaconStateView.node);

  // console.log(ssz.capella.BeaconState.getPathInfo(['balances']).gindex);

  // console.log(bytesToHex(beaconStateTree.getNode(44n).root));

  // console.log(
  //   beaconStateTree
  //     .getSingleProof(44n)
  //     .map(x => `[${hexToBits(bytesToHex(x)).toString()}]`)
  //     .toString(),
  // );

  // console.log(
  //   '----------------------------------------------------------------',
  // );

  // console.log(hexToBits(bytesToHex(result)).join(', '));

  // const resultMerkelize = merkleize(balances, 274877906944);

  // // console.log(bytesToHex(resultMerkelize));

  // console.log(
  //   sha256(
  //     '0x' +
  //       bytesToHex(resultMerkelize) +
  //       bytesToHex(ssz.UintNum64.hashTreeRoot(32)),
  //   ),
  // );

  // console.log(hexToBits(bytesToHex(ssz.UintNum64.hashTreeRoot(32))).join(','));

  // console.log(
  //   BigInt('0x' + bytesToHex(result))
  //     .toString(2)
  //     .padStart(256, '0')
  //     .split('')
  //     .join(','),
  // );

  // console.log('------------------------------------------------------');

  // const result = bytesToHex(
  //   ssz.phase0.Validators.hashTreeRoot(
  //     beaconState.validators.slice(4096, 6144),
  //   ),
  // );

  // console.log(result);

  // console.log('-------------------------------------');

  // const balancesView = ssz.capella.BeaconState.fields.balances.toViewDU(
  //   beaconState.balances,
  // );

  // const balancesTree = new Tree(balancesView.node);

  // console.log(
  //   balancesTree
  //     .getSingleProof(
  //       ssz.capella.BeaconState.fields.balances.getPathInfo([0]).gindex,
  //     )
  //     .map(bytesToHex),
  // );

  // console.log(
  //   'merkle length',
  //   balancesTree.getSingleProof(
  //     ssz.capella.BeaconState.fields.balances.getPathInfo([0]).gindex,
  //   ).length,
  // );

  // console.log('balances length', beaconState.balances.length);

  // console.log(bytesToHex(ssz.phase0.Validators.hashTreeRoot([])));

  // console.log(bytesToHex(ssz.phase0.Validators.hashTreeRoot(validators)));

  // const validatorsView = ssz.phase0.Validators.toViewDU(validators);
  // const validatorsTree = new Tree(validatorsView.node);

  // const pathInfo = validatorsTree.getSingleProof(
  //   ssz.phase0.Validators.getPathInfo([0]).gindex,
  // );

  // const lengthBuf = Buffer.alloc(32);
  // lengthBuf.writeUIntLE(10, 0, 6);

  // console.log('10 as hex', bytesToHex(new Uint8Array(lengthBuf)));

  // console.log(pathInfo.map(bytesToHex));

  // const hasher = new ListCompositeType(ssz.phase0.Validator, 549755813888);

  // console.log(hasher.depth);

  // const hashTreeRoot = ssz.phase0.Validators.hashTreeRoot(validators);

  // const singleValidatorHashTreeRoot = ssz.phase0.Validator.hashTreeRoot(
  //   validators[0],
  // );

  // const singleValidatorHashTreeRoot1 = ssz.phase0.Validator.hashTreeRoot(
  //   validators[1],
  // );

  // const singleValidatorHashTreeRoot2 = ssz.phase0.Validator.hashTreeRoot(
  //   validators[2],
  // );

  // const singleValidatorHashTreeRoot3 = ssz.phase0.Validator.hashTreeRoot(
  //   validators[3],
  // );

  // const result2 = sha256([
  //   ...singleValidatorHashTreeRoot2,
  //   ...singleValidatorHashTreeRoot3,
  // ]);

  // console.log(
  //   BigInt('0x' + bytesToHex(singleValidatorHashTreeRoot1))
  //     .toString(2)
  //     .split('')
  //     .join(', '),
  // );

  // console.log('--------------------------------------------------------');

  // const result = sha256([
  //   ...singleValidatorHashTreeRoot,
  //   ...singleValidatorHashTreeRoot1,
  // ]);
  // console.log(BigInt(result).toString(2).split('').join(', '));

  // console.log('actual result --------------');

  // const actualResult = sha256('0x' + formatHex(result) + formatHex(result2));

  // console.log(BigInt(actualResult).toString(2).split('').join(', '));

  // console.log('level 1=================================================');
  // const level1Hasher = new ListCompositeType(ssz.phase0.Validator, 1);

  // console.log(level1Hasher.depth);

  // console.log(level1Hasher.limit);

  // const level1 = level1Hasher.hashTreeRoot(validators.slice(0, 4));

  // console.log(
  //   BigInt('0x' + bytesToHex(level1))
  //     .toString(2)
  //     .split('')
  //     .join(', '),
  // );

  // console.log('level 2=================================================');
  // const level2Hasher = new ListCompositeType(ssz.phase0.Validator, 4);

  // console.log('depth', level2Hasher.depth);

  // const level2 = level2Hasher.hashTreeRoot(validators.slice(0, 4));

  // console.log(
  //   BigInt('0x' + bytesToHex(level2))
  //     .toString(2)
  //     .split('')
  //     .join(', '),
  // );

  // console.log('level 3=================================================');
  // const level3Hasher = new ListCompositeType(ssz.phase0.Validator, 4);

  // console.log('depth', level3Hasher.depth);

  // const level3 = level3Hasher.hashTreeRoot(validators.slice(0, 8));

  // console.log(
  //   BigInt('0x' + bytesToHex(level3))
  //     .toString(2)
  //     .split('')
  //     .join(', '),
  // );

  // console.log('--------------------------------------------------------');

  // console.log(
  //   BigInt('0x' + bytesToHex(singleValidatorHashTreeRoot))
  //     .toString(2)
  //     .split('')
  //     .join(', '),
  // );

  // console.log('--------------------------------------------------------');

  // console.log(
  //   BigInt('0x' + bytesToHex(hashTreeRoot))
  //     .toString(2)
  //     .split('')
  //     .join(', '),
  // );

  //   const redis = new RedisLocal('localhost', 6381);

  //   const db = new Redis('redis://localhost:6381');

  //   const work_queue = new WorkQueue(new KeyPrefix('first_level_proofs'));

  //   console.log(await work_queue.queueLen(db));
  //   const buffer = new ArrayBuffer(8);
  //   const dataView = new DataView(buffer);
  //   dataView.setFloat64(0, 123, false);
  //   console.log('Buffer', buffer);
  //   await work_queue.addItem(db, new Item(buffer));

  //   const item = await work_queue.lease(db, 200);
})();
