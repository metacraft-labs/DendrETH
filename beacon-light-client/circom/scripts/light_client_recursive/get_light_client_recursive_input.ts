import { PointG1, PointG2 } from '@noble/bls12-381';
import {
  bigint_to_array,
  bytesToHex,
  formatHex,
  hexToBytes,
  utils,
} from '../../../../libs/typescript/ts-utils/bls';
import { writeFileSync } from 'fs';
import { BitVectorType } from '@chainsafe/ssz';
import * as path from 'path';
import { getFilesInDir } from '../../../../libs/typescript/ts-utils/data';
import * as constants from '../../../../relay/constants/network_config.json';

export async function getProof(vkey, proof, originator, prevUpdate, update) {
  const { ssz } = await import('@lodestar/types');

  let points: PointG1[] = prevUpdate.next_sync_committee.pubkeys.map(x =>
    PointG1.fromHex(x.slice(2)),
  );
  const SyncCommitteeBits = new BitVectorType(512);
  let bitmask = SyncCommitteeBits.fromJson(
    update.sync_aggregate.sync_committee_bits,
  );
  let signature: PointG2 = PointG2.fromSignature(
    formatHex(update.sync_aggregate.sync_committee_signature),
  );
  const BeaconBlockHeader = ssz.phase0.BeaconBlockHeader;
  let block_header = BeaconBlockHeader.defaultValue();
  block_header.slot = Number.parseInt(update.attested_header.slot);
  block_header.proposerIndex = Number.parseInt(
    update.attested_header.proposer_index,
  );
  block_header.parentRoot = hexToBytes(update.attested_header.parent_root);
  block_header.stateRoot = hexToBytes(update.attested_header.state_root);
  block_header.bodyRoot = hexToBytes(update.attested_header.body_root);
  let hash = BeaconBlockHeader.hashTreeRoot(block_header);

  let prevBlock_header = BeaconBlockHeader.defaultValue();
  prevBlock_header.slot = Number.parseInt(prevUpdate.attested_header.slot);
  prevBlock_header.proposerIndex = Number.parseInt(
    prevUpdate.attested_header.proposer_index,
  );
  prevBlock_header.parentRoot = hexToBytes(
    prevUpdate.attested_header.parent_root,
  );
  prevBlock_header.stateRoot = hexToBytes(
    prevUpdate.attested_header.state_root,
  );
  prevBlock_header.bodyRoot = hexToBytes(prevUpdate.attested_header.body_root);
  let prevHash = BeaconBlockHeader.hashTreeRoot(prevBlock_header);

  let branch = prevUpdate.next_sync_committee_branch;
  branch = branch.map(x => BigInt(x).toString(2).padStart(256, '0').split(''));

  let dataView = new DataView(new ArrayBuffer(8));
  dataView.setBigUint64(0, BigInt(prevBlock_header.slot));
  let slot = dataView.getBigUint64(0, true);
  slot = BigInt('0x' + slot.toString(16).padStart(16, '0').padEnd(64, '0'));

  dataView.setBigUint64(0, BigInt(prevBlock_header.proposerIndex));
  let proposer_index = dataView.getBigUint64(0, true);
  proposer_index = BigInt(
    '0x' + proposer_index.toString(16).padStart(16, '0').padEnd(64, '0'),
  );

  let nextBlockHeaderHash1 = BigInt('0x' + bytesToHex(hash))
    .toString(2)
    .padStart(256, '0')
    .slice(0, 253);
  let nextBlockHeaderHash2 = BigInt('0x' + bytesToHex(hash))
    .toString(2)
    .padStart(256, '0')
    .slice(253, 256);

  let prevBlockHeaderHash1 = BigInt('0x' + bytesToHex(prevHash))
    .toString(2)
    .padStart(256, '0')
    .slice(0, 253);
  let prevBlockHeaderHash2 = BigInt('0x' + bytesToHex(prevHash))
    .toString(2)
    .padStart(256, '0')
    .slice(253, 256);

  let input = {
    points: points.map(x => [
      bigint_to_array(55, 7, x.toAffine()[0].value),
      bigint_to_array(55, 7, x.toAffine()[1].value),
    ]),
    originator: originator,
    prevHeaderHashNum: [
      BigInt('0b' + prevBlockHeaderHash1).toString(10),
      BigInt('0b' + prevBlockHeaderHash2).toString(10),
    ],
    nextHeaderHashNum: [
      BigInt('0b' + nextBlockHeaderHash1).toString(10),
      BigInt('0b' + nextBlockHeaderHash2).toString(10),
    ],
    slot: slot.toString(2).padStart(256, '0').split(''),
    proposer_index: proposer_index.toString(2).padStart(256, '0').split(''),
    parent_root: BigInt(
      '0x' + bytesToHex(prevBlock_header.parentRoot as Uint8Array),
    )
      .toString(2)
      .padStart(256, '0')
      .split(''),
    state_root: BigInt(
      '0x' + bytesToHex(prevBlock_header.stateRoot as Uint8Array),
    )
      .toString(2)
      .padStart(256, '0')
      .split(''),
    body_root: BigInt(
      '0x' + bytesToHex(prevBlock_header.bodyRoot as Uint8Array),
    )
      .toString(2)
      .padStart(256, '0')
      .split(''),
    fork_version: BigInt(constants.pratter.FORK_VERSION)
      .toString(2)
      .padStart(32, '0')
      .split(''),
    GENESIS_VALIDATORS_ROOT: BigInt(constants.pratter.GENESIS_VALIDATORS_ROOT)
      .toString(2)
      .padStart(256, '0')
      .split(''),
    DOMAIN_SYNC_COMMITTEE: BigInt(constants.pratter.DOMAIN_SYNC_COMMITTEE)
      .toString(2)
      .padStart(32, '0')
      .split(''),
    aggregatedKey: BigInt(prevUpdate.next_sync_committee.aggregate_pubkey)
      .toString(2)
      .split(''),
    negalfa1xbeta2: vkey.negalfa1xbeta2,
    gamma2: vkey.gamma2,
    delta2: vkey.delta2,
    IC: vkey.IC,

    // proof
    negpa: proof.negpa,
    pb: proof.pb,
    pc: proof.pc,
    pubInput: proof.pubInput,
    bitmask: bitmask.toBoolArray().map(x => (x ? '1' : '0')),
    branch: branch,
    signature: [
      [
        bigint_to_array(55, 7, signature.toAffine()[0].c0.value),
        bigint_to_array(55, 7, signature.toAffine()[0].c1.value),
      ],
      [
        bigint_to_array(55, 7, signature.toAffine()[1].c0.value),
        bigint_to_array(55, 7, signature.toAffine()[1].c1.value),
      ],
    ],
  };
  return input;
}

(async () => {
  const UPDATES = getFilesInDir(
    path.join(
      __dirname,
      '../../../../',
      'vendor',
      'eth2-light-client-updates',
      'mainnet',
      'updates',
    ),
  );

  let prevUpdate = UPDATES[0];

  for (let update of UPDATES.slice(1, 2)) {
    writeFileSync(
      path.join(__dirname, 'input.json'),
      JSON.stringify(
        await getProof(
          {
            gamma2: [
              [
                [
                  '5896345417453',
                  '4240670514135',
                  '6172078461917',
                  '219834884668',
                  '2138480846496',
                  '206187650596',
                ],
                [
                  '6286472319682',
                  '5759053266064',
                  '8549822680278',
                  '8639745994386',
                  '912741836299',
                  '219532437284',
                ],
              ],
              [
                [
                  '4404069170602',
                  '525855202521',
                  '8311963231281',
                  '825823174727',
                  '854139906743',
                  '161342114743',
                ],
                [
                  '3147424765787',
                  '7086132606363',
                  '7632907980226',
                  '5320198199754',
                  '6592898451945',
                  '77528801456',
                ],
              ],
            ],
            delta2: [
              [
                [
                  '5896345417453',
                  '4240670514135',
                  '6172078461917',
                  '219834884668',
                  '2138480846496',
                  '206187650596',
                ],
                [
                  '6286472319682',
                  '5759053266064',
                  '8549822680278',
                  '8639745994386',
                  '912741836299',
                  '219532437284',
                ],
              ],
              [
                [
                  '4404069170602',
                  '525855202521',
                  '8311963231281',
                  '825823174727',
                  '854139906743',
                  '161342114743',
                ],
                [
                  '3147424765787',
                  '7086132606363',
                  '7632907980226',
                  '5320198199754',
                  '6592898451945',
                  '77528801456',
                ],
              ],
            ],
            negalfa1xbeta2: [
              [
                [
                  '4063420080633',
                  '6555003798509',
                  '3528875089017',
                  '5800537096256',
                  '8041381108016',
                  '203518374640',
                ],
                [
                  '7676269984398',
                  '1145806392863',
                  '6738515895690',
                  '5144301275423',
                  '8547057760405',
                  '353834589854',
                ],
              ],
              [
                [
                  '5712635615088',
                  '8763475698695',
                  '7480760495871',
                  '1630925336586',
                  '5902994417779',
                  '229051200835',
                ],
                [
                  '1066113280330',
                  '5452941452156',
                  '130670027992',
                  '6364438679415',
                  '8227984268724',
                  '117895881848',
                ],
              ],
              [
                [
                  '2720638156466',
                  '8183746692879',
                  '2805734624200',
                  '4541538633192',
                  '1476702149455',
                  '162434980571',
                ],
                [
                  '4093955238700',
                  '4839352246179',
                  '5773319594517',
                  '5269728708172',
                  '8404179905859',
                  '4522318692',
                ],
              ],
              [
                [
                  '7907150524416',
                  '8555524456643',
                  '2425990496019',
                  '5117607179458',
                  '886559720121',
                  '343845114320',
                ],
                [
                  '3348806304058',
                  '5295378168489',
                  '5426585403009',
                  '4313512356362',
                  '2882006508456',
                  '312905790371',
                ],
              ],
              [
                [
                  '6984987484510',
                  '4411212100320',
                  '517962775393',
                  '5578757090043',
                  '1344911245314',
                  '115782940661',
                ],
                [
                  '4257694794763',
                  '5641455412912',
                  '2987387394488',
                  '6147130513016',
                  '8766894161060',
                  '7451503335',
                ],
              ],
              [
                [
                  '3338043330865',
                  '3023333978926',
                  '4787719622265',
                  '3729967781503',
                  '2489094582823',
                  '396043239802',
                ],
                [
                  '3390886416082',
                  '169102433935',
                  '2279828268438',
                  '1618451670976',
                  '7055320302964',
                  '48334526481',
                ],
              ],
            ],
            IC: [
              [
                [
                  '7865676620781',
                  '7085838017712',
                  '5763238399198',
                  '2824046590194',
                  '8565721526276',
                  '234374434407',
                ],
                [
                  '4409343437197',
                  '3347034602193',
                  '4094100413004',
                  '2967874303211',
                  '7685636634493',
                  '36051760889',
                ],
              ],
              [
                [
                  '335546885105',
                  '4341428588987',
                  '4970373224509',
                  '1014576301323',
                  '5842790174473',
                  '390870955229',
                ],
                [
                  '2973440947494',
                  '1270824899072',
                  '7610226271513',
                  '4922349606938',
                  '1975595330088',
                  '188198353488',
                ],
              ],
              [
                [
                  '7261373843462',
                  '1466375881697',
                  '877673687623',
                  '1350749075759',
                  '2873201522702',
                  '183767888021',
                ],
                [
                  '6743558513818',
                  '8622590278229',
                  '1114980196978',
                  '2396752396964',
                  '8259315090001',
                  '62361696638',
                ],
              ],
              [
                [
                  '5170338054463',
                  '157639936973',
                  '4117875637741',
                  '762484732208',
                  '7587423276049',
                  '250170253614',
                ],
                [
                  '5196647758738',
                  '2578630167193',
                  '8276538375152',
                  '2737126282772',
                  '7683478528439',
                  '384147675102',
                ],
              ],
              [
                [
                  '3016153973354',
                  '5937702456273',
                  '2893838271698',
                  '8455587427335',
                  '1160206087526',
                  '237450009011',
                ],
                [
                  '6425376769888',
                  '5742073986731',
                  '5079018036738',
                  '8411416797464',
                  '4164233093738',
                  '196080607854',
                ],
              ],
            ],
          },
          {
            negpa: [
              [
                '2664335738635',
                '756967847293',
                '7479395195527',
                '4041805873448',
                '1913427542084',
                '186020833398',
              ],
              [
                '6161538689603',
                '5807793715572',
                '1520934201814',
                '3284889581503',
                '947891306159',
                '27309946865',
              ],
            ],
            pb: [
              [
                [
                  '4619969552706',
                  '3470329841725',
                  '5091681317545',
                  '647337064543',
                  '4730286203560',
                  '371074480034',
                ],
                [
                  '729366282247',
                  '3873126821603',
                  '4032672539614',
                  '927392462659',
                  '8350738538988',
                  '412711721953',
                ],
              ],
              [
                [
                  '7669567465857',
                  '5349733182057',
                  '8005356594793',
                  '7754624174835',
                  '2504853479348',
                  '296898128775',
                ],
                [
                  '7451508092292',
                  '5958119656101',
                  '1773933026886',
                  '7132396355436',
                  '1070703123032',
                  '36772226445',
                ],
              ],
            ],
            pc: [
              [
                '4331002993308',
                '1362521130494',
                '28216148123',
                '2048675256150',
                '7893125126976',
                '344763611822',
              ],
              [
                '7874451291585',
                '2964148932401',
                '4692768793178',
                '6480370687379',
                '3829197219626',
                '200328504747',
              ],
            ],
            pubInput: [
              '5966029082507805980254291345114545245067072315222408966008558171151621124246',
              '4',
              '12857343771181087157409557648182655546684462713036905539892384468792366321123',
              '6',
            ],
          },
          [
            '5966029082507805980254291345114545245067072315222408966008558171151621124246',
            '4',
          ],
          JSON.parse(prevUpdate.toString()),
          JSON.parse(update as unknown as string),
        ),
      ),
    );
  }
})();
