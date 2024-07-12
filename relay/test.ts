import { PointG1, PointG2 } from '@noble/bls12-381';
import {
  bigint_to_array,
  bytesToHex,
  formatHex,
  hexToBytes,
} from '@dendreth/utils/ts-utils/bls';
import { hexToBits } from '@dendreth/utils/ts-utils/hex-utils';
import { BitVectorType } from '@chainsafe/ssz';
import { sha256 } from 'ethers/lib/utils';
import { DenebClient } from 'telepathyx/src/operatorx/deneb';
const fs = require('fs');

import { buildPoseidon, buildPoseidonReference } from 'circomlibjs';

import { numberToBytesBE } from '@noble/bls12-381/math';
(async () => {
  const { DenebClient } = await import('telepathyx/src/operatorx/deneb');
  const { ChainId } = await import('telepathyx/src/operatorx/config');

  const { ssz } = await import('@lodestar/types');

  const denebClient = new DenebClient(
    'http://gpu-server-001:5052',
    ChainId.Sepolia,
  );

  await getStepUpdate(denebClient, ssz, 5239744);
  await getRotateUpdate(denebClient, ssz, 5239744);

  // let rotate = await denebClient.getRotateUpdate(5239744);
  // let finalizedBlock = await denebClient.getBlock(5239744);

  // let pubkeysBytes = rotate.nextSyncCommittee.pubkeys;
  // let aggregatePubkeyBytesX = rotate.nextSyncCommittee.aggregatePubkey;
  // let pubkeysBigIntX = rotate.nextSyncCommittee.pubkeys
  //   .map(x => PointG1.fromHex(x))
  //   .map(x => bigint_to_array(55, 7, x.toAffine()[0].value));
  // let pubkeysBigIntY = rotate.nextSyncCommittee.pubkeys
  //   .map(x => PointG1.fromHex(x))
  //   .map(x => bigint_to_array(55, 7, x.toAffine()[1].value));
  // let syncCommitteeSSZ =
  //   ssz.deneb.BeaconState.fields.nextSyncCommittee.hashTreeRoot(
  //     rotate.nextSyncCommittee,
  //   );

  // let syncCommitteeBranch = rotate.nextSyncCommitteeBranch;

  // let syncCommitteePoseidon = getPoseidonInputs();

  // let finalizedHeaderRoot = ssz.deneb.BeaconBlock.hashTreeRoot(finalizedBlock);
  // let finalizedSlot = ssz.deneb.BeaconBlock.fields.slot.hashTreeRoot(
  //   finalizedBlock.slot,
  // );
  // let finalizedProposerIndex = finalizedBlock.proposerIndex;
  // let finalizedParentRoot = finalizedBlock.parentRoot;
  // let finalizedStateRoot = finalizedBlock.stateRoot;
  // let finalizedBodyRoot = ssz.deneb.BeaconBlockBody.hashTreeRoot(
  //   finalizedBlock.body as any,
  // );
})();
async function getStepUpdate(
  denebClient: DenebClient,
  ssz: typeof import('@lodestar/types').ssz,
  slot: number,
) {
  const step = await denebClient.getStepUpdate(slot);
  let pubkeysX = step.currentSyncCommittee.pubkeys
    .map(x => PointG1.fromHex(x))
    .map(x => bigint_to_array(55, 7, x.toAffine()[0].value));
  let pubkeysY = step.currentSyncCommittee.pubkeys
    .map(x => PointG1.fromHex(x))
    .map(x => bigint_to_array(55, 7, x.toAffine()[1].value));
  const SyncCommitteeBits = new BitVectorType(512);

  let aggregationBits = step.syncAggregate.syncCommitteeBits
    .toBoolArray()
    .map(x => (x ? '1' : '0'));

  let signaturePoint = PointG2.fromSignature(
    bytesToHex(step.syncAggregate.syncCommitteeSignature),
  );
  let signature = [
    [
      bigint_to_array(55, 7, signaturePoint.toAffine()[0].c0.value),
      bigint_to_array(55, 7, signaturePoint.toAffine()[0].c1.value),
    ],
    [
      bigint_to_array(55, 7, signaturePoint.toAffine()[1].c0.value),
      bigint_to_array(55, 7, signaturePoint.toAffine()[1].c1.value),
    ],
  ];
  let sha256_fork_version = sha256(
    '0x' +
      bytesToHex(step.forkVersion) +
      bytesToHex(step.genesisValidatorsRoot),
  );

  const DOMAIN_SYNC_COMMITTEE = '07000000'; //removed the x0
  let domain = DOMAIN_SYNC_COMMITTEE + sha256_fork_version.slice(2, 58);
  let signing_root = sha256(
    '0x' + bytesToHex(step.attestedHeaderRoot) + domain,
  );
  let participation = aggregationBits
    .map(x => Number(x))
    .reduce((a, b) => a + b, 0);
  let participationBytes = numberToBytesBE(
    toLittleEndian(BigInt(participation)),
    32,
  );
  let syncCommitteePoseidonBytes = await getPoseidonInputs();
  let syncCommitteePoseidon = bytesToHex(syncCommitteePoseidonBytes);

  let finalityBranch = step.finalityBranch;
  let executionStateRoot = step.executionStateRoot;
  let executionStateBranch = step.executionStateBranch;
  let attestedHeaderRoot = step.attestedHeaderRoot;
  let attestedSlot = step.attestedBlock.slot;
  let attestedSlotBytes = numberToBytesBE(
    toLittleEndian(BigInt(attestedSlot)),
    32,
  );
  let attestedProposerIndex = step.attestedBlock.proposerIndex;
  let attestedProposerIndexBytes = numberToBytesBE(
    toLittleEndian(BigInt(attestedProposerIndex)),
    32,
  );
  let attestedParentRoot = step.attestedBlock.parentRoot;
  let attestedStateRoot = step.attestedBlock.stateRoot;
  let attestedBodyRoot = ssz.deneb.BeaconBlockBody.hashTreeRoot(
    step.attestedBlock.body as any,
  );
  let finalizedHeaderRoot = step.finalizedHeaderRoot;
  let finalizedSlot = step.finalizedBlock.slot;
  let finalizedSlotBytes = numberToBytesBE(
    toLittleEndian(BigInt(finalizedSlot)),
    32,
  );
  let finalizedProposerIndex = step.finalizedBlock.proposerIndex;
  let finalizedProposerIndexBytes = numberToBytesBE(
    toLittleEndian(BigInt(finalizedProposerIndex)),
    32,
  );
  let finalizedParentRoot = step.finalizedBlock.parentRoot;
  let finalizedStateRoot = step.finalizedBlock.stateRoot;
  let finalizedBodyRoot = ssz.deneb.BeaconBlockBody.hashTreeRoot(
    step.finalizedBlock.body as any,
  );

  let sha0 = sha256(
    '0x' + bytesToHex(attestedSlotBytes) + bytesToHex(finalizedSlotBytes),
  );
  let sha1 = sha256(sha0 + bytesToHex(finalizedHeaderRoot));
  let sha2 = sha256(sha1 + bytesToHex(participationBytes));
  let sha3 = sha256(sha2 + bytesToHex(executionStateRoot));
  let sha4 = sha256(sha3 + syncCommitteePoseidon);
  let publicInputsRoot = getFirst253Bits(hexToBytes(sha4));

  // Create the JSON object in the specified order
  const jsonOutput = {
    /* Attested Header */
    attestedHeaderRoot: Array.from(Buffer.from(attestedHeaderRoot)),
    attestedSlot: Array.from(Buffer.from(attestedSlotBytes)),
    attestedProposerIndex: Array.from(Buffer.from(attestedProposerIndexBytes)),
    attestedParentRoot: Array.from(Buffer.from(attestedParentRoot)),
    attestedStateRoot: Array.from(Buffer.from(attestedStateRoot)),
    attestedBodyRoot: Array.from(Buffer.from(attestedBodyRoot)),
    /* Finalized Header */
    finalizedHeaderRoot: Array.from(Buffer.from(finalizedHeaderRoot)),
    finalizedSlot: Array.from(Buffer.from(finalizedSlotBytes)),
    finalizedProposerIndex: Array.from(
      Buffer.from(finalizedProposerIndexBytes),
    ),
    finalizedParentRoot: Array.from(Buffer.from(finalizedParentRoot)),
    finalizedStateRoot: Array.from(Buffer.from(finalizedStateRoot)),
    finalizedBodyRoot: Array.from(Buffer.from(finalizedBodyRoot)),
    /* Sync Committee Protocol */
    pubkeysX,
    pubkeysY,
    aggregationBits,
    signature,
    domain: Array.from(Buffer.from(hexToBytes(domain))),
    signingRoot: Array.from(Buffer.from(hexToBytes(signing_root))),
    participation: getFirst253Bits(participationBytes),
    syncCommitteePoseidon: getFirst253Bits(syncCommitteePoseidonBytes),
    /* Finality Proof */
    finalityBranch: finalityBranch.map(finalityB =>
      Array.from(Buffer.from(finalityB)),
    ),
    /* Execution State Proof */
    executionStateRoot: Array.from(Buffer.from(executionStateRoot)),
    executionStateBranch: executionStateBranch.map(
      executionSB => Array.from(Buffer.from(executionSB)), //9 not 8
    ),
    /* Commitment to Public Inputs */
    publicInputsRoot: publicInputsRoot,
  };
  // Write the JSON output to a file
  const outputFilePath = 'InputForStepUpdate.json';
  await fs.writeFile(
    outputFilePath,
    JSON.stringify(jsonOutput, null, 2),
    'utf-8',
  );
  console.log(`JSON file has been written to ${outputFilePath}`);
}

async function getRotateUpdate(
  denebClient: DenebClient,
  ssz: typeof import('@lodestar/types').ssz,
  slot: number,
) {
  let rotate = await denebClient.getRotateUpdate(slot);
  let finalizedBlock = await denebClient.getBlock(slot);

  let pubkeysBytes = rotate.nextSyncCommittee.pubkeys;
  let aggregatePubkeyBytesX = rotate.nextSyncCommittee.aggregatePubkey;
  let pubkeysBigIntX = rotate.nextSyncCommittee.pubkeys
    .map(x => PointG1.fromHex(x))
    .map(x => bigint_to_array(55, 7, x.toAffine()[0].value));
  let pubkeysBigIntY = rotate.nextSyncCommittee.pubkeys
    .map(x => PointG1.fromHex(x))
    .map(x => bigint_to_array(55, 7, x.toAffine()[1].value));
  let syncCommitteeSSZ =
    ssz.deneb.BeaconState.fields.nextSyncCommittee.hashTreeRoot(
      rotate.nextSyncCommittee,
    );

  let syncCommitteeBranch = rotate.nextSyncCommitteeBranch;

  let syncCommitteePoseidonBytes = await getPoseidonInputs();
  let syncCommitteePoseidon = bytesToHex(syncCommitteePoseidonBytes);

  let finalizedHeaderRoot = ssz.deneb.BeaconBlock.hashTreeRoot(finalizedBlock);
  let finalizedSlot = ssz.deneb.BeaconBlock.fields.slot.hashTreeRoot(
    finalizedBlock.slot,
  );
  let finalizedProposerIndex = finalizedBlock.proposerIndex;
  let finalizedProposerIndexBytes = numberToBytesBE(
    toLittleEndian(BigInt(finalizedProposerIndex)),
    32,
  );
  let finalizedParentRoot = finalizedBlock.parentRoot;
  let finalizedStateRoot = finalizedBlock.stateRoot;
  let finalizedBodyRoot = ssz.deneb.BeaconBlockBody.hashTreeRoot(
    finalizedBlock.body as any,
  );
  const jsonOutput = {
    pubkeysBytes: pubkeysBytes.map(pubkeys => Array.from(Buffer.from(pubkeys))),
    aggregatePubkeyBytesX: Array.from(Buffer.from(aggregatePubkeyBytesX)),
    pubkeysBigIntX,
    pubkeysBigIntY,
    syncCommitteeSSZ: Array.from(Buffer.from(syncCommitteeSSZ)),
    syncCommitteeBranch: syncCommitteeBranch.map(branch =>
      Array.from(Buffer.from(branch)),
    ),
    syncCommitteePoseidon: getFirst253Bits(syncCommitteePoseidonBytes),
    finalizedHeaderRoot: Array.from(Buffer.from(finalizedHeaderRoot)),
    finalizedSlot: Array.from(Buffer.from(finalizedSlot)),
    finalizedProposerIndex: Array.from(
      Buffer.from(finalizedProposerIndexBytes),
    ),
    finalizedParentRoot: Array.from(Buffer.from(finalizedParentRoot)),
    finalizedStateRoot: Array.from(Buffer.from(finalizedStateRoot)),
    finalizedBodyRoot: Array.from(Buffer.from(finalizedBodyRoot)),
  };
  // Write the JSON output to a file
  const outputFilePath = 'InputForRotateUpdate.json';
  await fs.writeFile(
    outputFilePath,
    JSON.stringify(jsonOutput, null, 2),
    'utf-8',
  );

  console.log(`JSON file has been written to ${outputFilePath}`);
}

async function getPoseidonInputs() {
  const pubkeys = [
    '0xa7ecfb69d8c08ee7c4155ac69adda7393593e6614b349cf83e07586a2b3fce780a54ecf31f1536b35f428a4f75263ac0',
    '0xa1c78c5cf35f1a5b398ce01a7b2b3333d59886e68e257d8eb06a3e893456a8b2d201a5285a0653285bdabef409a0859a',
    '0xb1a62050ac40e852adfd2e5903d3b62e12bf5b8310222e0a4f9eef2ef1d7a0ad0cdc2c196c81578345038e7bd5525712',
    '0xa3a661f075c844111c98c62c24dc25ebafded96689ad4f40e4d0bb1365aac01098c90f7f76f6f7e01663dadcd29433d5',
    '0x95d58695031adceaab18a40d0c8d5bec74de110353563a597a29bc1f22b45ea06d3c2358f8d9fba013ca7c4cb8c489dc',
    '0xb9608b4c007152afad1ec7cea1cfde62c1c0dc6050aba2cb8dcf65e47c8579a46f2c73cef1fa1cdc12cb6f8f0c7a3096',
    '0x8faab76819e9747b00ae02bf16deb3efef697fa421463c26962a2888c3ff65fa8eab808216622d773d49f9d2639fd1c5',
    '0xa54c5c968ad4fd2095deb55ddc48a4d630b0ac6aaf410585d71bfb95c1f81df5081d44d04c00e9e2e54b071041284d0c',
    '0xb9886aec63c7ac721b5d2f29474b43b7be946bc1eccecddaf0a9aa024234166e8a0ea999c6cba8a13e182c3fb5a097a9',
    '0xab3c8e9f662c0aa040a3d1b1548f7ade804ea5ef3443ba4685d25a5ae3263292665b7d24eeb1719f21946c194901f580',
    '0x89b933e4b70f49b27c597013df36c4c3069ab1e4889833262641bf5208528c60f9d85db7030de4022dd9d759d49c21dc',
    '0x809625a743635d165c2bb5c24e57615913587316d13e3fcdc9e74809e6c5f3f206d2a7e45a3d771fc6bd16a0bd30163f',
    '0x87f30b710bb224c420ee424084721642a0d246f74916d2fe9f1a2b66491eaa43a978ce4ce6400999ea9585b566f2c292',
    '0xb9fef955c495192a341337aa793fd7bf81e32539cf58f4de6e8e74e1e0bd05d565fb970871f521069ebd9929ae0c71cb',
    '0xacc455734027b751284b48fa41aec6af20492ae2682b8db87bc4a2f6db8895bbf1237beaaa2b4f07737c3c503727c639',
    '0xb614b56e3426f752707e7319117da909932393a77294cd46288dc040d2263089c7b371fbf6d58a7c9106414df608d67d',
    '0x9809a0954633b7ece8552068ff885e2c43cdf0ede7b0d1465e7f5d97b57e5e3feb743d2383c7b10c8b89494416f7b818',
    '0xb1db6051d254c4b04caf2301e652ed7661c16b51c473bcb5c7425554d55539ed085b527580e86f847f9e48424fe78a2f',
    '0xa1b85e7f9a70db05fda44a0d3548f81e467e562643637376743628bbcd1b581e7415a5c821a9a412dd31af5ca097aafd',
    '0x855590c6bf3f3163e3c309230426342853085d6ff930c6f192cd113d0454224a40a57dcc87fced0e63fd19a0fd8ef627',
    '0x979dd102d52506add7c469edcbfda9dcd2710e97b0a2753e0d04f05543150b7b6044b62ce752c4117a905883183a6bb1',
    '0x905d0ab5458c471760275e426730b577865dc0eb60bde227f535aefee9f2d53ab0ed78a1ba40cbe9aaf94cfde932589d',
    '0xa2695b79b31e0fbb84c0a897346a02b4fb5e99ef394862bea8b6f8c120fff79a8da9e2d6f3e526897c8131fcd33fbcb5',
    '0x8da3bfcf3620311ba5ddedcad67480d759ac7bfb7fe07c73d3f33ac735de3b6e01b93012721ba4bc8785da45c47f7857',
    '0xa110a4bf93cf89f151ffc7c51de0e323911bb7249729cce87a51f72b39d42125d16b4dcf66227b2183680925e1d0b17d',
    '0xb304b19328c8bba80f7e9e45e47d97f13313572bf38392b8a69e04238033d22f888b4602ef882d66eb70f7af68ab120e',
    '0x8814693cb6ff176779a8154f933d9fc948d33f675cbbe6b3f897cdba96408a5ddfef0c4195dea5ddd29b33da8d16d139',
    '0xb3369dc545e831c683e35973235e6d11c5e171a23f057799d8bfbd3cbcc74d84de75ffb76c266ec90dc4b19ed91cd47d',
    '0xaa1a14bbbca3140437bb2a1b26b38a85d544fb5509ac0d6a808fa85c36c1d4dc41474426b592887acc1db806c99c3142',
    '0x803a0066f3c07e62289ffcf8d734f4f9341b510c5daff5ac184adc642044ab4fb5e4e7cf52b0d2f33123c98df4486496',
    '0x87eaecb3ff33ba05259f7f26eb7de7fe802388592e3cf2b96e9495f4940baebb9aaa6a2d65c329d173c7c368cc51d154',
    '0xa69df4f5bf4eb8a1a4228dee5586cccb7191cf87064fdfa71ea0388f604f7e8d144ff14358f4b47cea3fe402fb2a781b',
    '0xa776dd0b4516eae4cb83b90fa2f7610e7f70cfa38f1b6d89778dd1f0aaf4bc24e100ca9a07e1d049575f42890ae4a9bc',
    '0xaa8831258b8d650ed6ddbcab5ec60a152d7e40e99e9c68dccc07ccb8fb6bcd0937508566263ceb3d7e778dbd924eb111',
    '0xae7eed21aa3ad2fd85133d0fd2e547aff1af6b7dc7ef037af34481a09b4d89278ca9c39cf387661454a4f7251fc7291a',
    '0x98069e3f211c134344ed5c0b0131eae27766cb6340d04e62c0b759c16fbbae041554f2f975c6e3f21c4954da85bcf345',
    '0xb10dcce866d6545751056c10a5f69cebe21e32084f6d5e00fe994be4ef396b54d0234af974235e7da20b2b9af9b600b1',
    '0x8e6df7831acc6283e9a760bc3473d69976f83f7f837bd624fdcbd471303b0035380511896320d68ada06b9e5bf5cc67b',
    '0xb8e9a7e674c7ad25c929227ad17def8a9c033f7c67ccd37b4450268a76565ec851d5dd834f478321ae1d1b04fb8f595f',
    '0x85c35af630154ba66468cba8fee3548b9602c076214a11fa21b5ad2d7ee40b27a54e0a8525740720ebe9d968442a8537',
    '0x9442a21edc2a06da568c3814915d54c8bd91cd8bb9d2d6d0f5662c1c865e33c22e04e59e4f63d255dbe87da20eb539a3',
    '0xb6838846d31a3c7a559dd74a7a781c9c8e956d94b0be459a6dddfea70b0f922a72821182e67f9f656180a162b00920a1',
    '0xb08c46abf95f370da1851a3129c57434f3d8a9a1b2ccf4272f8bac47f62022b3c03f22304867ba2a4ee79eb57a5b3a1f',
    '0xa585a898a83520ed595951c0bbc13ab7eae33c548c1f97c977f0626e68865f4353baa4b8bf22771c1e29f6f7ce9bdc37',
    '0xb81233ba1f7ae6889453664d3f497ece79eedf476520573205b68bca8fbc438ed6360db538a8a9612a2d550c65bbb1a9',
    '0xa56d04ce75ac7aebdce6b6cce5e03a7dbecbcd45f4fb296d494ae3913e73c00d03cc0ff63c1dcab02b78bd9e6b92fa61',
    '0xb14d8fc073eab5249b24b7bfc588e07c5edb9e22aecac7af1433ba8589bc978292729816b7b1745caa9d30762595b3b9',
    '0x87cab3e94b908fa3c6e98d8cc1fe8de7f0f28547458d25c6e70428c99568d69117cf5ceb898406df7ca3693d3d5d6eaf',
    '0xadbc0715d258f9dfd58b6ef564377c9c40c7a733c74a8e06af99075242caf21ef9e62325a0fbe77062c2fe7a46ac3373',
    '0x8a20df03922ba15e3b2f0ddb772f260d1cdf6e939f46c6d02dcd72b445a0c1dd1f0b281d9e2a80be01de1c839d3f0b10',
    '0xab42e5525fafeb75a0fea8b4a7a05848dc9df4232c170774cdc3ba34dfe486741536a201f737abe0cf6f70f4673d6310',
    '0xa1e61efcabdb7797ce3d340b54299689df8fca2f22992d3757517575a1da63eb8c23c61aa84c388ed330f6cf80568080',
    '0x88576d0a10db85c5a79c0d25312f670d15553afcbef48d06db9925973adca0f0824390d8b145d07be09ceb2bcd31e525',
    '0xa65549ca29b941051408b268d8e12d0c2f3f1893bf39168ac4987393e7c72489ce29df3886a9e47d8e2459eeebf58d00',
    '0x8d9a2ac0b8d3d87232df8edf19b27cb3951d10787c73f5a37cb724d435096f45c7f316b25f1848ff1e72ebe2a8b6897c',
    '0x89febbcf9b7fb32a47c0f7f5734e164675053e49aa0d4dd8690a5e670212f88bf4ee43af1da9a18149a224f20236db23',
    '0x965266839e9ad9c72646aadb3740a1367523ae7dc7423a23016f268a7b5e578a70df94b8e97e8c7f9337ccee3e88641b',
    '0xa927d75f44a59514a841f7c6a9372490443c6e8a1d3a7a82ab2796632d06383ba694f80cbd789c1823b7ae14d7f8aaad',
    '0xa1d73a22ac28363dbeef00b2bebc5efa85fb2f9529b7a4b08c4ffe113b0a932e54c55d78d9e2eb54e4805822aae2c0a9',
    '0xb9c39c0713e615c514008a4daf2fb2979412b66e9705c5f90f49489117850af37e9af7ea6496a68797813343b0e9d24a',
    '0xaa49d6b000d9eaa123bc7bc4e9b6a7d18e17d7c267c8a10e2fcc20f9b1c12f3af75801cf3fd32b9e09754bbdfcb84be4',
    '0x961213e1349afe1af8f022fc38bd38b0955caf504dccc37615c99cb36e7c526f4d1aa3c74d48e3243afdd4fc6c393135',
    '0xb961d570c5fbb45696695468ecfb90ceca9244cabcfa27db9058a37caf1e480cc4e5770eeebf9e22ef0b5337684147d7',
    '0xb437105afa9cbf407e5279e16165e123f933c0fa0e57b0b9e838cd7addce785c53918e41a14472e814ef84a0a823237f',
    '0xb5ae4d2fe28e773b2343cb3f03b10c9fde60885f7565adbae047aaa3cdb1170071e0a09bd904c0f81ebf30142cde1650',
    '0x8bfb5c7c67e189644bbb9fa57e8d96125b596835dcb80afe7d76704839e1b8e9ce590400c5c8fa38f90a588721498e0b',
    '0xae04d730e09f4ae597cebd7aa24e2e095d2629c176ab0bd6c143eb2aa346fd981c74fa94153780d63181accfc4185b8b',
    '0x8a19c96bb0bf0cd6b3c6b8a6b5a4879897c460dd8955ed44f714ced72b894b1e1555adb214759d0e2c4ffcb0258e2875',
    '0x8ea039bda17de0beaee128062fbfa0ace9e7eeb03d447a924743e8645e07e240cfaa5988c1d7184d6a66b733fd434543',
    '0xb63267f4dcaaec8abf5dda58c56b7f6e5256d8d954ebf3c0b65f744a3f7b80f8c580a5ab65527c2b88f90368f3077313',
    '0x93433c788f913b9d554e6432a613dbf6fe962c2714901783d884bf6f2ac02b857d41429517e793cec468b844b684232e',
    '0x99185441fee42ffff49a084ce4fb576ecece82136f5f86ffabc72c19f8a2886c72edc5b97bf83c25da0dbd7bf0187c2c',
    '0xa8646cdb63360090f2d45cf2272a2008209f62c45f9c378ba96338441a281a85d90463a5f0757e7d99ae9a3e2ce1363f',
    '0xaf265c57ad52f9b34659eed1e093309899f6a222a5b605df24723342e49f337d80d5692af160ab09ddbfa4a5fa960bf2',
    '0xb54f7a599628954c79125e11827efc6dd02a77a0e5ca1a7843e122b169424be46e4dcc1e0a155345052dee4021757b26',
    '0xa354e63cec7fdd579d78e90f94421fdc283340201da2e5833788ed13e72b81364e40291b7bdacd31e2aff1539254fd90',
    '0xa7ec9ec6f4f0d94dcf596d5bec9b1dd5651ed6294c67daabe6739a587eb8f3918d22fdf41bde5bcf4efe8f31567d2e67',
    '0x915d2e0ca21df950a8e353a9f1b33ccb865d42ad2238fcbede7567cd654e3798de9a408a0af98c08bccf967d5e780057',
    '0x8504e324a128eac77e6e04f3254ca4d1d511ad119e179b7ff0a8ce37e695e3cd11c589c970f114b5a598ab9d7d3b33b2',
    '0x801bafc742b80cca59e725eedf870789bc8e6027f7496b5f3f3d51564a1f9debf95775bfc390e09808b3473c5a313d23',
    '0xb7ef52758335009c845e506831b0686fc25133d1509fa004fff9d4c035562e30c8075fc33d696dc1f0b7b64929a3b9d5',
    '0xb06a976cab91555e2fc48be574a2f514f0923c3b299916c561bfd32fb16e3fbc80fc6e1e9a27ce119336be9a333e9178',
    '0x95520c8a6883f3dc4f7da8cdec70515ac59fa39a6660851fe4c5edb6daea64f9a4ba4ce98660a08237363ecdee049529',
    '0xa48d7b13a4dac25b36720fcba17678c0c9996b94e3e0f14eba70393414dcdc1c017b35c865f128d2e060935808a8c925',
    '0xb209485c2d7b598f602ce7e6e70feac9d5c83a961717f290fe5d072683dcdd3a846b0bd2d9935b75a0fcf177cd52736f',
    '0xa626c24dc5561e589a4adffdddca5a56bed673f03d2195d59ac4787d522e1244e7ae6fa62609ea203211e82656e3b865',
    '0x8539177b2b5090a5e07e42f2604832c5027874c1fcd40235e947fe472e60c37977f3b09b9d576ee2d7697418ce54822c',
    '0x81e1e930b94478a22633d593e7afd6e37833bc14175d5336d2688c90e42a270a67ff45f8b8537f01dc9b01a5f791b95b',
    '0x916efee62b19510b41adab587918c054269596af273dde146ed5e522a0129e0974e425e4d963686da7b1cceedc99f11c',
    '0xb3427f0f411297858942a48da7cd2022720040059425f2b41a66697ee171c0bcab0c79d2ccdbdcbc801ffff410cd1c97',
    '0x91da4170241c3f21bb018df7c479ebc537dee7a592349fe4a41b8a8f4a87b10b224216ee00e1bdc7b8d98e1da2197b9d',
    '0xa72acca628fb905b33dd64e6171b969636b94886aa84ac386277ce5d9210f523a17f0028ec13ec0c052edf125f6bfa20',
    '0xb81d11f7e43684e9b45c01cf289f57d0d4ec0705441558e849ce6691774ea7db647e64a8061dd57479947ffd4f8d16b3',
    '0x8fcd6043ecce255745910ffab01d6a38f40fe8e154f2832925f95ef53085767474e1c34385097ac3eaf7764fe6d6e0bf',
    '0x8813828e523445ca4182e0c76d9286ea7d0d286b427a7b5d4585917e48032763d0c86fb55af2ee64b447d61a9628ca8f',
    '0xaec0dbeb6ce1be16a4023b8713000bd254898950feaa7d28cc04c673f55afdd77b0d10f7b8f3b30c0a0602127f26f041',
    '0x8745b7c87b64a8a1f3766cfea968e28dc9cafa0602d7a16d697c42f19c2ed8f4f612273e2d8c394afe974077ebbc9a1a',
    '0xacd05a545530be73a87328804445f1616d2b4053a8d111d609ec3ed707bf0c767555e0d1bfad61561bf87cf4d553b384',
    '0xa92b37157131a50be009053a443e6b13f50749a690b7a1fd5cc7c74b5bb587467d50065ac045c41d60f088175783ed1b',
    '0xab8b2bf1d32140047b0ac1ea2d5a38f7d8e4cada27634d485dd3ed57e97811baadb83f8e9cffba525af72894090d079a',
    '0x90286952b43c5e1f58c2ed55da258a24d289a3bb36b5c3b7a69fe62bcb48206c52a03fb44da026e666293f02485917ec',
    '0x95659fef30f86a07f6c96d08cd6ede3f982499128b2a4ae1153d0a443fe90c29086b385e4a0dfaa587fae930b7c83db7',
    '0xafe92296f1aa7812a59ae87af913b713a827c153757d67cff50e920650eb042a6784293e5ee90f1948007ecfd0410de6',
    '0xb7d8ad2e6ab6eb52e0d747ffca64d7d213c59cb48c722e155ee8f577196b287d85b3c09fe849d558ab174dd92be942a7',
    '0x8fa9d8c181b547f70242c54c798b3d59db34f043fc02cc9c47d90ee937779fa24512fa2fa84fdaa95e412a1389296ca0',
    '0xb369e271dec55e25840de82951e854e79e28b69be544248da9ac439866c7bfd210cf8cd462eab4dd95b945a04fb2ba04',
    '0xabf8447460813caba72615c7a5ea39e85f124e2b4ac2997e8e3261cb77b631327c033e857f9b2486b2769e619e3e06dd',
    '0xb5a6904024329435b717808bdde4258a770d21ba764c1c3ed9027456134425884cd7fd179e5c33196221cbfaeebaedc9',
    '0xad9c247b91ad5c502af3e5a1fec611c5a833774dcae9ce8a9280ef0ab4707022ac134c8ffb5bc21a5d71bf840ba6daa0',
    '0x95ece4b6a9c0545c917aa81fa791df5498dd298f18146ab861b688baae1a01577d82ed8cb1fc68430c27b571cfe21f7e',
    '0x837de5b04b7c2b53e4ce8895ae027a046c1efc1ecb7acea110a55651927cc2e9008580bcfbe55980f6bea557bcba426c',
    '0x93a6a2273e704dc61c34e140e9f73bec97de603c9f99f9280ba425b467ac90cccadf77c299cfe636b2e3c7b91f09ebbf',
    '0x8301475e95df7a2d667ae89fe775f0e24928d5d704896adec30457e28d9f17c44bb99f4cc78876e4b58807b2322ddc21',
    '0xb33882895dee079d041b60385b8902700ef9d3d4e3912d9f8d258e7dfbdb3a2b2245cb7d61dce73226add96f8d0a08b6',
    '0x8f7a1d0987175a06fc664302415c70a61a4f6d2c47a7d24ed602d1ed5249e121696cf30a1e6105f8b6b4a4a9f63b382e',
    '0xaeb3867c56cf59e38067051b8ddf72cd6378c9c653988b6825ab10c97148a5c099e1bbd07e782654b77450174ac5c532',
    '0x921f2de435ef4cf0bca150ef8a8a51baa20feb4b112b6db318395ec60efb2dc5b3c5fb58b7c0d1d64aa6830bd23e4139',
    '0xb4b5cb9038729aa33ed958a9fe1ec93beff74147c13abd255d9f0c9baa1011578c77228e6e2df29ff66c90efb7989a4e',
    '0xb5ad3fca5a7def3f121b14337a7b9a9d7bd471ee888e8242ba06a6634475534109b08baabd2f8e48fa68392a813dd3f2',
    '0xad261153732ba85ffd7340bc969e7a8131781f00a744da507147b353eb7121809226a961da86d32e58dcfbcb0a5d1cc3',
    '0x84e391c3fec90740ed68a772bd404cc76b6e826841f08a5b35d41209d530bd5cb98ebcf85ec9019cb6941d3ac4f10555',
    '0xa7331cc453b3cb22ebd81b5bb46b4b303fdfa7798d8e50d875f9a9bebbe344c55b2ec5200e3d71a56bcf48ce9e6073e1',
    '0xb3eb72b7b165f9f9427f8756797865bb6154d0a51d157aec166c72dd02f56282f40447bad664cf573e172500ced2143c',
    '0xa767690ee0d2d7693cd1d54ff6e393a9003f8ed75f16e22393b3f3a5f25ce4a9529859024b1fd13268fdbaa538d58716',
    '0x84ba21e4b99544d0e5c2ad547c57e85b71dac328f3cde16ff6a9ce987a658211ae767bb50262e0d9a0737a05c61ea8f6',
    '0x8da6096a8a4f8781b6c774057e2ebf447980d08d76239c2ca0f77530ee2f71694a10eda05e4bd7839e6d84c8d3df4dba',
    '0xb0704c4ebff4438a184cfa8e9fd08d0aca68cb10a9b531a760aa2556a1c7f82fc6226b0508977c07ed563929fee5ba3b',
    '0xb7aac4c198dab73e1e7469dd2cf49a9e2bcf34d1dffa88add2d26e7c916ca4765567c31c4079f700b3c82ca8bbba413d',
    '0xb8920d2f84118fdfac7e208546e73e9f8cd0a4d91ebbd7fe04064d9ff3ea40e82bc87eb1c47dd36a055305d8cdc3a73b',
    '0x827107ab1c8581a34982b9fe1222f954421496214314a389639e8742d11003c573abbe62149d8f727b03196e3c53a302',
    '0xaea548e03d423ec4530a1778be9fa6044dc71aaa59ae35ec341bb5ab719448e093c4315596dc95d4e85db16f40262a4f',
    '0xb3f0b8cf0b5a7c6ebcf0c46c1be6d3c1d0a5a8ff5639188b3854206f2912f384e9f4d13fe1de35da1ec0ff4cc0917c56',
    '0x8c57d767a77d092ae49e68ce4d61da33795a12c5fe7a101d61029a63422e34691994ee6e4568fe66a5ffe268ec29cab2',
    '0xb01aa5183c0aefc8ec1b1b9c2a6f15ab4e8c5bf24381b3df095b4e3b17305152c7b6688834c28bd7aeeb0faef94990db',
    '0xb5226db6d2fc129c44ae9245e7ef5d6e9814a985ed3e3858c8503e5e54bfabb21fdd5bbee2ef52d1646b4aba7b452e8c',
    '0xb53baa1fa7afe9def32e619722e1585f21b2df71904035a077eb7346f07dc6b10d1514ad297b87ab3dfb5c7b0e157f95',
    '0xaff8b53e952d17bad84192aa6baa1a098c1b6e2bde6899532f7f74f2c1f4f7cefa0c603c4ec1964c0841aacee63005b8',
    '0x80af8b4de0c5fc90fa4d1afdef2550aa42b1b4441433e79d8c5dcf4842835642b8414d496c1d8bd7e684606dc177aa86',
    '0xb2bf0b39c037e50da76af8eeb437784946ec8b5e2bf6a2e4223d428b7c9c56c129105ac7cc4cbb113e97d510e9e40d49',
    '0xa637fa2376e0d28b259df310d451997bcfb71d14a3705afd6dfb2df738d7b1d9c86b116d0d46efd189d5c4542ce16a62',
    '0xaac325b4aa2f3847d6681ff21b6cae147350d0ec0ea7f87189c459c5a1adf77ccb291680fa71a35f5505fe059711190f',
    '0xa69b8677878d992b4bf5d60008430ecaaa5dec531e29c74c9bae8f3a41bd5f776fe4a4da4f3d3b2e3cd29941bf4326dc',
    '0x90d865e669debe8f748174ec06550556f029dbf46566aaba9d6bc1f42443a954d8abdb5b456ade2c9603ba8634d7a080',
    '0xa8b1f88a088665305f094935a47d01de56f6e92369bef6c09b7df844912a8ddda7ae3397d46bf7e748a5761b1254034e',
    '0x8d38374c88f2794738f8aae1d1ea0b9eec00c6c0e3d23896592095e4454c6967f8af72727c3d92c774e450ed15285bc2',
    '0xb8b72d0149bf2b0690ba5da64fda9093e30e54bf28e3faeede8d7730dfcb21ee92709b66195be897b4f566689464b10d',
    '0xab7ba61ecde87f846985900176cbbce41e14a486f3dc2411f15b1bc5f2f4e13bec6a3f03d37d1605d3045ee64f99e74c',
    '0xa670cc03f8ea150bfdb5bc2ce032a082d4e3321aee21d259f6b4b6e4dcdee5e25bc4d85b4cd8706184e8f0736ea4811a',
    '0x8b716639c9488d234d6f75be41fc8b391e0e337cfc897120315b8e598496b6b5648c1e99270cf3802af2403506b512e2',
    '0x8d0f5edeff96150d78a94b16eb7c62d9416362799bdb15959a1cb09a51a37c2d71c9fba310ddfa278c20100fc7402ca5',
    '0xa363e8ef8b62056902a4c5e115137f40a257ab4cce36298eb63b443fd4fb07f54e92a3f6aad24070c5df014ddb192a0c',
    '0xb269f58122a8239c318792f6442a513a8ec341aa9ed4134c3a8d55bc456fcfd6056bd20d24df43a36b66d637e4f0be2c',
    '0x82b8e3d1532521495c1de2e894e929f8495f2b92289baf71f231ac9e730e607a9a0f0813386cc81c9dbc1e936213430c',
    '0xa9b4d331b3035b7ee8a7a08451f719c119cfa770d595d3ab6ed428e74824c941ec8c9fe5ed1368f0e6da9c86002e7533',
    '0x97b4cc403762793d69341dc9c4d73c605e27822ac980af1ed4b9b585235f9de2e37b8a27f4ff8a6e7132cae6ec9811eb',
    '0xb1bc19ab1d313d1030a2eaadad923c954c8c94d2a7885be304aac9cf7013f283e798b4f94fa8250d1da04898c0c2ecae',
    '0x9221b26b7849c7a46a2356347f69096173e69e19affc4da3e78f72cd7bad80c08cd47c82b4e47967cbb867b8a2d251a3',
    '0x8cda50dba6735f901575372afc56e3f68b2bc589db3119c6439b269435369f0fd9e8f61b53f50bb092bb74aaa0441a45',
    '0x91ec93ec425a5f20606bc92de63c609c564bf6a379cd0effb364d7c2f65e708613111c2d32d6bd2bd8ee9bf97abe09cb',
    '0x99975fc0d1c40b03379a9389a920a826dba5ba33a3012ae8d2dc209c2cf738466c4d1f6bd38f6cf1cddd79cf9cb2d854',
    '0x84baf9ce459c279ce1b83b536f060536309404ab4e59f32997017d8f1462832d311682a8eab4210d24a21260d295a3dc',
    '0x958603bcf583584e5cc962ab28550d1db048fbc832d7ab298a8d8919e965b8fb99c96a02532cf27fbfb843a917673192',
    '0x89bb9c0dad6b94342742f4df50c18d6b4d5bfe3bb1068a31dea464bf78b4ccd653f4d5cddef30d04f4102ca934edf645',
    '0xb992f5cccd2808874c4af63042a2eafcc224aaa0de34ada24b2685e58f62542e3c7442de247b1edd5a5024687f364f16',
    '0x92c45c13da7932e8f7d554e11aa93b72d5fe5e7041f3dbe2d561ddb21b569a3f5a61a8bace8c643dcf1f1e3f0eac9fbd',
    '0x977364ad36a7c40a8af3369c1e3501609bd6e40ad4d41c2681b8ce0f09f632f8b3a1451cdccb898f2f53f59cb26afdef',
    '0x81066321e617349553cb625ecb25b437a26c53901a20270961a68889759442d6edc0bd92493531d67b36bc7b96a44e26',
    '0x8512ab9cec4b11c928b1b8d2a98906cc4a6a7c2e8aafc53368726445c08f72ff325d60f468c5039915dbc957dc812003',
    '0xa812aa9f6488c6efaf3dd1663972a0a150b2dff09ca0c5fb4e7dfe92e1aa4565d92a55b40c8b4d97048f580c589280e0',
    '0xb9dea0c965afe151cb18abc4215d71511ac6b879ed4e9a0901af1398a5177010adee9c491491572a27027ab5e834110b',
    '0x8b756ec20371db2c974ffc237051bd05ec0e2462b77068787d0c274a899ff71c38041d9d210698726d769cdc2233b91b',
    '0x829b29e912b95ed0eda1a1599b14b3335b4bc1adf2b47a421074dfa837a158c77f95e250426e9e2446dd376782806e35',
    '0x89ace428a54a4295b7a5f9e34e0c6977ec87dbcc03ebd579a8d6e6f837fd87deeb01a198d91324bb01dd075dc42c40d3',
    '0x83aea8f9f83d685620f85afef4afce2db2b2fb8e5fa890d5760f314f4eae8192257c01fcc0b79102c93a0ff32e103335',
    '0x943359e119dcbf7c77f5cee703962dd718fe7fd0760ac7bd537d8cb33e16f42519ebdb8132e915444331df4a789e8a46',
    '0xade3035a36718807990258d2e82701acc510573361573fd93c88f3911ea10390f3162445a64fd62a22290404ba1a9f8f',
    '0xa5607cb3181e552661842cf130c4e1b8b53ebafd442072895f076754fd7f3634931aab49b7c7c0a04db6c70220782f4f',
    '0x899db285759cec35b5767c45760f602f919a7e46ae5ba1df8aa1cc32c6dea2542dea1663fd773f5edf36a80a215cf07a',
    '0x96dd5d596d5f4bf7140eb5d52b1b7ca0e4eaf0747da71039c9ecd6470bc0593d0b86adaf782a94a3b37014fae1540335',
    '0x9964f7576189d3c41de490d32093a905a8f4e599bff4b0b79f9351ec20f89d3796519c61f395fac6d97ef68ecac2d957',
    '0xb2497048a45bf025372991903133d15259be1b2934e7342ab1aa793ed20a1fcdae4b6978323f4f9cfd508b167a13c0df',
    '0xb447ab2c3fe7b653871427f708de9a6a364ee182cb22cd9d9cf12e112a82a1a112c2ac93d90a83dc03166217838ab4ab',
    '0xb947b77560769df086c2520b7a9c557935344ef183e3529754d59c97cb9c6483eed67829368383382b7352233e18831c',
    '0xa7266c5159cc8f5d8e5316928df980d944b6ffb3c35fbe03359756edfa6b25024c78afff8b8016f07c6b246148c07093',
    '0xb7f230e7e7391aa2af48f2847bde2ba25b9b75ec458d0e758ef4c499776b30cee40052ada3fb96a098b328b9f33a20cb',
    '0xa6b3ae8db6a812dfde01db4803eb3266b01f53349cc371e9c9c908a300ccac4ec7a8f268d3c00bec42c54b3ee5734e94',
    '0x854ef3be432693dadc62a3f45dbc3a7e6e1d9374aa0834f6afb7611e72b2c90d276c994f58c281f7e23fbdd9405adad0',
    '0x81390854ea24c54f80337530b12f2a0c1acf3df39ef4501c769302fdac869034cb7abe5d09b3b00f4535477591a1a675',
    '0x81d35e817a947141e41435506658ccbadce4f6fcc5c7ba4e48f05d33ff230eeca87a1d342ceaf0c783501adab8ea6f93',
    '0xa011b98f3a54bc01347d4cb6bb8d518e67165fa98e5c231e75ceacc57140978d8ae92eda31c14bc4bd10b99b7d936fd8',
    '0xa5bbc658e5df21d667108682f2dd1555df422053cad7a63182f5127960704d0af2c5667c5315ded2bb4498bd38776344',
    '0x8c124fbb7ab141b8ce085012dbcd2414c7ac3a1480c251ca0f1c4e49d13ce19923286f594e43a22cb1bdc016dd964e99',
    '0x8c5d4c14c130b2a829e2922bf4ed0c093146b04054ec7996291f37bfbaa70cceafbb4c83a7bb5c4dc761bd2e39f49eaa',
    '0xa41761aff2455623f3cc488e25b9c312003510b50d47d0b95d9f2ced101e97c8f5d26426566e9fb65c76f6fa8a809ee2',
    '0x9276c09831cf14549543072f9dd12690049e091c1d0b74124b86e5702c146782fe42b7261232367d740dd26e9f2a0985',
    '0x8637abf3d5491267db571ce323ea8d4ab5f9bef9bfb8cf76b48384870e57c094b4524ee366d0f4eda330360da1b8674a',
    '0x80f7c48494c8600d38ed9ae35a48a1ded5c6ff4a9e39e9400461d4f823d08f955264adffe95d009b8d8c4102738cb29a',
    '0x8e5b689bbb079a96d868621b5dd973c6fa7fd0f48b2808b4d900934eefc02d3cc0a5e860a7c8fc4fd42e7ae86ef1d80e',
    '0xa4dd3ce0cc499febff407657b241a84f9288d8579b482e68771a11f3c1aaecda34b3cb3a5a1e5a4067c55885c3883e2a',
    '0xaa00d638bf9d9736132ab328e2e17abb50e5089a39b29f66288ae83e9b36a33fb72ad0ca2a896ddf5c3a920a2d203ad6',
    '0x98361727703e7af4578e146f46f9971e1ad59a95a1c082be10baa2d7a258c0ca9d8227b743e300bd2cde6901db0eb9e9',
    '0xae98f025e5f79675b018602c71195ff750cf590959966b279b5c5b2f2164d8bd4f9df3c40d7f15653cff161f9a522578',
    '0xaf1649eef1394a05d065bad97dfb349dd3f2898630dcb0732d90e3aaf58eb85cf4c0e03ffb75e900319c44a059c0aa29',
    '0x9337d4a69af4f69f103df3d95c1df34ae7ae8faf713b661d482c35fd90712feb4758e77d34674a91177f220940ccd47f',
    '0xa48d2d02159c10f5959c1703c33e4e542a2495b31541154671c050246a4a8d90f6ae1681cca068a5bd7391069c56bd14',
    '0xacacb27269f74ae8bc7b2a8a7201868e043ef4923e036719b2151dad1d0dfab4a146d167093909b5318445fbf12a8704',
    '0x8088854b1baba95fa5c9b4b8de7dd09405b307d4c9cd13aef2374b04c1caab70cca1766463fa8ae30990df1c2dc8c7f3',
    '0x93bf04b32423cc35a3def8e0ac398dbbfa0a4fd36c3ce8a474598b1c03c60efe8599fd057663601904d508f13b6081b8',
    '0xb3630b23f3edca57255d4dc98775ee2ecfe7b290a2922651cc8ef08cb8176a5eb7417e5af357e7eddc568411761fe753',
    '0x8d91c4d315b5af01e9776da94c5f24e20da26d68241a60d9f3d150c98f69a1441fc864a9a6ed353731308b2ba47e32c7',
    '0xa0d91c1571dda86e84ec5dc4521e7e2361b742d0cb5b9868334a0d8772f671b5c9b848262e2bbc285f48bb46b09ddecf',
    '0x83d8d0ab68e98f6684f248e171777b83187b8b202879dc32af4ddeb320d1999d9cdf91fbe02bf3998556d9f102e42cab',
    '0xb27db98f239e7f1081d90cf52f62248f582c765218a7031c260aaee580cac5ecd9360a249e36ca8c9f7f557bd33f5a9c',
    '0x95e5b1c3d9aca60467cdb6859b83b2b6fd18dde2daa0198efa8bbef46308180aa9c240cd978746bf7fe05861aa9ba3cf',
    '0xa0d72b8db62ea29f0e009805642dfffbd080165a8003ef694aad770f584db49e0f6f7e98d43ec9a03c274a336978c180',
    '0x8a4934d6baf61c780fc4de110f03830138d4281b1e70c9b9f38c481e85c1866a4a413609e3eaae67d0ee93c08efafc18',
    '0xaa42bc3510ea892c0ee45f5d64d6530f8096aab463275809bbdba691d7d2a40b11ccecca1a67f5090beed7a030196551',
    '0xa5bc4501accf6bad5738f61274a6a44bbdcae0c03da6d4be75c0c368d2ccad81f6492af1ef0159d9a1ef56633df64078',
    '0xaea180c02af017bf9cacd3ebdb8a072c09ae33715cddccf9b61e4cc8c4736578322be33cc6d2396328373bfc515aaf68',
    '0x86a000bd655fefa83f6a11343ef20a779eb91853ce5b30ddc4ad79d6c4f4eb580ab7f639f9b6f1cc91001bbd6bf4f627',
    '0x902880e17a93ed9e63529b5d77a1ce54041979dbee4d9a74d3288688c4f3e5613d85f52d10031a6be7a9b781f2e7af56',
    '0x828e1668b3cdeb47efc7e79defc031bfea413bf36fdb84aed29232cf383f71c7aa067713f9ce6d607d52722be91b1937',
    '0xb35de1f131dcbb1c819a25060a26969ac3e0cdc3d3107fab6eb0eced2a6c8bd8b26616a7020673693bbeee7adef51939',
    '0xa1c1427b976a518199ca40edacc0c81842ea1c5391d43fd9edd77d80f1c242d4e3f90540e1f5c0eed96430094c79bb8e',
    '0xb26979b4df27d307fff6f046b5a7edd5b730cd9b1f11bfe327e54070724a547c60f61dee2a176ef84046eebbada55127',
    '0x99b6013c71e3444acfb7943b85d4ea69d5d044342bbf686cea28e9223be50235cbb03b4f58ed91486e8189f452805c35',
    '0x91d11b3244cf89715f07c52e8dc40530e930b19d4e5632fbd9e97fb66c47eee4dcace5332f18992bc886793aada76080',
    '0xb6abc1de966a26fd393760cc8fd85000d44013c023410fdb9340eee60ac439990964b16f307e569030f2ac60b9cd8464',
    '0xb7531fc6846e16c5950362cece96f172fa39cf8dff71e37b0ae142f74b25828cf91bff25ff91687977dc13e875942c6d',
    '0xaaf043a07c1faffa7f449da656e25c936bf9c054c50e75bf27bcab00b1e3a8b49eeaec2cb0e18f14b74df9f339b00bbb',
    '0xa59c7158dd7c0e6dc9acb1dacf354dd5ef5a98c2d35043858de0058bdb764e82d10e63c6212937401091a4af3c12a340',
    '0x86348644d4a677b8e8697d56db4d8d2026aaa5030b15d52e09e7d5471c35f22a711290e4da1d95eecf997fe97835291f',
    '0x8eb83c2cfda770165eca6101fcef122038e6cd612378251127933babe8aa08e7518204055b21f6fb2fbca8cef954e1df',
    '0x88ce2bb626d9bc844c32ccb85a2e336ebcffde27171b233d18d73902830f84d18b6afbff8880d9975ca89cc017d4529f',
    '0xac61adb1c783bbde8697e7a06fec1985c756c0d6596c0ffbb47a39e09aee163353520f39a8f1393101e0cc325c406d51',
    '0xb6e529609613cbc7135946457462f88c5dcf2c0fdf279ceff19c375a6f4e1fbcf6a9d9159b3b2651f9261dfa71431cc6',
    '0x91711f6a7d8f49a25738fe94ce596e3dc65c52cf3a68cfa2c128efbec4a750cc6d92ace4d2f6294f1990d38b123ba6be',
    '0xa141a1bc6055b8e1c2d51c6f413d4dd5d8746f27c23c4d46bfa392769c568d0ee5d43ec2b4d3f2b6ff7af296436aaad2',
    '0xb787d9c7412447cc82dca8d7e0766a32af4255b1c9d02bebdccdd624a05a12d2862a1b88780e3289a2ac6f100e36d19b',
    '0x8385e25e7f26fb68b2c9972c6979e558898593c164236c39471a0d3895d2397646e2cb6b3f6131ac3d70f54223261737',
    '0xb2fac1258912f8a994b0d50ead59f0acac5ccaa5e265031e63a4ee90c7190666e304493a6c4095fed6a7381c8a63e6df',
    '0x898ded9a7c070561a9a0ba23816ece544c24db01199e216fc62b5c2bd92b2de291dcbe7c388ffed44a0be70fa25d5b48',
    '0xad69dc5be8dd032a80f6a989070bffaa8a83239b97d4f9da27324dac99fc4d3b40f37a29234d0f92b1e4a2713f4add0c',
    '0x91d49827f35e21ced920f958257faefa7ab50ed8d64fb579c8dc80296f31315b0a5aa6bf77345c96434510367aea50e8',
    '0xb40efa12b5aefe42b82e43603a0e94f04684fd82d7378a67ca0b4d80115a01ea80d57b4cb8957624612b0a2378d0d28e',
    '0x8b6782714925132072fdf9c9a8b875540b54826566a9487920bc1147467a234a07a5d8afd92f5d78c9b83081c58a21a2',
    '0xacf4e15e5b1311f2f5f3f2d4d076736303fc3270d341182350e5890ab45e7a2a6d9dc1d52a08f9cdb7b9b87b66fdcc69',
    '0xab6d036eb1404ef4ee3dc07a21b987836c891a3e4471b9b4ef4aa2b41bf430a50ef0b517f2da0a667b665d4ecdad2da3',
    '0xad96976f8cfa5d7ec2f3aa49ed880f1ed300b08024c2a7c5da2bb3680e0b39e49798ab8c7bdb4a516d39e47043e2af94',
    '0xaadb4779a2239cd612d8cfe95eea7a0738d6f08bf2a55856c25b52068c5890f6b5f7db233ca64ed76b442f263650b907',
    '0xaf506d1fd447002f969600e28b827b9d385b7bea0c62f299ea5750c1c74693e12264ccd64d147c4c89b0b8cdab2b7816',
    '0xb11044200de9920a5d742769488dcf68c1bc6a8af07813813b69f514a30d85016c0195529fbbe0df94374ad3b5196b05',
    '0xa6d53a3aefdc26f43ce0898eee2f694104105a1d88e28b829b4dbe523100bdc3e8e9dc3c4263de8a9dec8c673fd55b26',
    '0x9137dd48ddebb81d8e2db5797a394417f04c27c39d8a5cd778d2588737771cc8b69c0c8825c31bbd45fa1f98f5bcb244',
    '0x82d1701779fb5cf44382ebf2704411523f50dabf36a57db3cc359aefd91c16e8c143a590aa5789ec1d7468d3a6a4af04',
    '0x957d6f46af7fca0876f0a0654958e26b3e3221d9b2061e60356a640fba7b6bfea04a2c92683c705bb335a2d375843a08',
    '0xa4e61d5fc856d7398303c77c9923b254a7b4396450e4a657be3b2f2cf49496935175564beb5d3305d1378e739a12dc1b',
    '0x88583c650c6cba4be6869297f3196eae4e00a2b16abb27d1632327e89ed8d208e7495302723e48f2b32eca55aee7385e',
    '0xb169a4e8e0a2e8ee1d282ed45b8a62a76d4bba4c3511be9883fa9e9b027f81a7d35cb3b6e947f850b8e2308602038682',
    '0x87ea52f0fcdc2685c8699275d623eaf85984b88b58a16037339f9c7773f9497de0294f72315a0ccd06ce1b9cb3a0f08e',
    '0xb71fd686fec66023594253731135052e8aba2637296b0ce00991a714681bdc1eb15964d03c8a4f79c9733e2825a2fe18',
    '0x88bde14a48eda6e56d2229cdc7fa0b99cc9d41e1c92834587e26a0ed7c329e349d415fbadc902cfc168599c78872a659',
    '0xa351c0ec14b7f8e80317a4ea2043d61148b519640ddb8541b2b833f17aa281803124391aaab357c3cde23b4bda28444a',
    '0xa6dedf9dbd36880fd418a1d15b1567e396fa3503af8bfe1633ba9047bfa046c5c8153452aa2ea5b22810784ea6a4b53c',
    '0x8044886deda381c84be777491a21e498f2bb2010f3ae9023a727cdfd519de106f389ce9503fb526b33547da4b8a3a216',
    '0xb1eee310f535d513e9ef4d4422c62fd97e3037a9e84f8159b9245aa81ac26c85126c6422f09d213d6a22997fbaee41e8',
    '0x90595ef80eb3d05fd497a627d492340044230dea16b760fdb2f70f8fcd8df3018e840eec8b271c96889dd70768f16f8d',
    '0x83805aa3ae1c643a57844127305dfee0d62b92019478a871b7d70ba330d2928e835699f9807ed7fec5248351b89e16cf',
    '0xa57615c261affff60fdff7eb25d420d44ec8e5fc97c4b51b5bcc511942edd2952874e818b0747ef1a0a1a13c0f6ae22a',
    '0x8c984903f46f8fa4ac3f0d4a4007fa370264910ceed5d7b97e20f70ab09b4988f08b3ae154f5ddb2f17615e6a268af13',
    '0x884d53fd6f627826ce87b9588d3eb9991546c3c4437c67b8aa8cf825def93cd795aebf11a4aeb856cdc02be56f7ecdbb',
    '0x904fcfc0c19b91a6a06df2ec28de28160a48b5fb7e0d1d8babb60c68f2a8937d38aa98564399bf6f1bbcc0264b321556',
    '0x8af30f5a3786bf84a7e30acd326aca60ba86525af4f83453302b76b581bff23c46def10318690c2aecc4cc57f92ed503',
    '0xb9d087e41ce61f287f5960e718d60366c2102166c5f3f2676ed111bcb80c815050846261c1d410729bb2d2b5f5bdd130',
    '0xaf006423c639c538909b1e94316272a365663249a122819ec395df604bfa8d10a5dcba4b685d043bac47e82322764c99',
    '0x99187f23c3f6f1854cf3a20db0384414cff8003bfdc007e111097be05a58725781b971f3af388a53710a263f79b69a87',
    '0xb22482fe61f2d4a9fd192eaac6a64bf35efc03a4bd6fc9d798623abc90403ce5ccd741781d8546605568ce67adbf870e',
    '0x93e2ab91fe872fdac66b6b8cb46acbc516107c4b8a20ac47ad9a3595ec757f0a95b1eba77d7ac3f4fd3a4ac552dbc414',
    '0xa2f1e310cc3d3d545eee5eaf3ebeb6a245772aeef8c3b43d93fc9a9d789bde8c11575ed3592bd3c7a62f292af34e9e06',
    '0xadcb73bfaadbeee9fd5adc2685bc158533897929192fd0716ab020051ef20fa5658331ea3b5ca32091b7d521cd07e4e9',
    '0x8544d486e903c1a3fcfa941b1716ee495cca26859a299aabd21e895f0ecc03fb4dedb1fbdd03dd492207de730083bbe6',
    '0x839fceb5204e9cd09a11399422b96b4f6d525c12bee4b4012def8f269d1b8a32348956d550c537179ad1ba70a0cd50f4',
    '0x83e7262d0534303a4792fa427bc2d250d9f9194a9b1fc03e320ece0e346acfc8fef407aba9d1b235741cde04f2956ca3',
    '0x813c2fd11b277ae0d2f356c1275b477f533c13d28a7b4db1b2459d2bde04f4b6a966e6519ce252b705bdbb4c6425aa93',
    '0x8fa43e91dc88fe96f0be1c95933ba2e0bf5143c979dae285e8f9910a55f5502a68a05023a2337d0adc58ddb566253658',
    '0xaf82e056aa86b6b6471f863ae2f0a326fbddde2a3d8d7c15b1ef5ed7ad2dc346e1f0f3709c5a51c798c678780857f9a5',
    '0xa1ef5ad999d1b3665de4cd5d516c8bf8cb831ad52344af359268d317d4abcdb3f32e804189a8c091871dd5a418cfce61',
    '0xb0d20f3894ef9e50e09b9dc8f09fa2e6f1a8e40ed0c0d6904fc38766b1f118c9cc22ec6609a73695390f041ffe55aba5',
    '0xa339b67d7005933f9c9662d7a13890addb0f3036454eedfb98069e68a6497011d2caa0ca1d90dd3886115c2576f84755',
    '0x95da70d6d31ba77373f339ccaa15aa72d06dfd121f6b503fe584c60dc042147ade5b97e3fa160639b5fa283f222fb2a7',
    '0x8285b535ce82a45372b804b78d70ba0f576628db6b91b3d46bcc2633835e10b5c5db8c031c1fd45dd6947d1972a61121',
    '0xb1c92e8cef3df1131da4292bd531ee01c403d123e333a07537f874df3f39f5a60dbd8504aa7d2ca3fc60576f3228bcc7',
    '0xb96256a52a79c4ab8de2197f1097f8264fa33248eb0e6e5defb6a35a87d45ae813fc36549850d2346e69d9c465e7023b',
    '0xa9f3a90def84fbd594c985605c8ee9d147ea63d48a6cb9983aafd8d01c0928257591babded8be80c28808f9957554ef4',
    '0xa098f47bc51fe064bff0f563fa30657eab673eb2745f6dfc89620c26efe17ff24c12a3332f985c248bce33f1003bf7e4',
    '0x818aeb08383e368fcfab1e3cb0d5125e3d4ae488109be8e4f02e3d8bff98893c2bc59f32b6dc59fdd8297ea50f7c56d3',
    '0x97099e064f856635bf0ba38368d752bad5fffdcecdded2fb69b857e5e93caa3fa957dfc43e482729355ae6f2a8ab3947',
    '0x8a4504848a67a9301c2c0b3057414a3858a8ead6d90d499597e320157137320be926f342e41a3b918e568978f9b2e625',
    '0xaf9832a02587ae93e1d859f2380457230b87ac2c98aabc853e09babe6840beb37a5bc367be70796c0c591b5114ecffad',
    '0x8ee7535b27d17cec9fde691dc2b0e14776454730ede1b171edd91f80b0f282e1f3b777eae183544d51673778b5873c55',
    '0x8a720b2e3e7764a866bcd453d3b25f0ec2c1f3e3320db8dc37d4913893b5db766c42c310310379b1368c91304521f8cc',
    '0x8feb9b1af301d9a2a221449babd0c9b733256bf44c75a4a598163349b0d14b9dcd24bddb868468c89e35bdc563f6f134',
    '0xa05df214eacfab4cd721a767ce5f4965300e28afe72f3b36e03646fade0b77c4649818f777678ab750f89d6e20165e8d',
    '0xa6f42dce90c9b1c6aa608b56c6f8f5f88a3439d5574bff2b28a07e3fae459bb1247bafebeca2d47cc6664db171c7cdf6',
    '0x8b05592adb9c3ad43ae35193a8f589aebddda0cf0959463da6eb19a5e9937446c1a6482910c78fd30d978b1705446b3a',
    '0x92aa736a0c840dcc3acc5aab3383d71e932629c6707799d2fd525af81c34739b5d38b4cbef1b3cc81d9f41235d458406',
    '0xa93f4da028f982b4159958b2456b4d0717d0c76ca517c543eadc34b6c810f28f662408d0b62094f6e6f4aa9137169d52',
    '0xb93e376dd66084b512dc87a15353bc957c35babc56cf5cc89dec818b5c64aa4628665e1aaa574695149307b7b8a5e0ad',
    '0xa04c061ee89c4eca6bf3057767425adc6f65f5da0588c19491ccad1d08052d165ee56aef6f4b095db9f3a1b10464ab8b',
    '0xa141fe566dd296c411c121542ba90d602e73a75140445e404acfa452e211124899a2bc8abd443a8be33ce43454b89ff8',
    '0x8b8803d67dede88acb36c6ac39063d272f56d7cc4abae2d898929240c10c3833c3b00925ac05a7524731b320b335c318',
    '0xa2945230a7ac97dce4a7dd6d6cde191ea044e7b888c1260d8e6cca4748719087a3c4e5c2a13af2b19ac67457f5aa4b68',
    '0xa2b17d55bbd9bb5398d7de2bf5bfda950ecb6036c2db0b5e561bcd8573aef9503e74c7906c37644ea1a17c4c39404042',
    '0xb54427e316998f1a59958f0d66bc491b1b87ec569d5d5ab0db5c26d2438977b432283e7ec19dcb2fd88b253d4f850c9b',
    '0xb8d577dd93ca0e8f57a71f30f9e0d82d03cd4b2e2e921ed691810a02c541f225ba6498c5b8afdca5a60e98c8e63b1b62',
    '0xaa99efa7a360ccd3c75a3bb43e1a315a59e7000c8168e677a4f4318aa493262e61ba4540e4c87dd8bb608e11af8ab1ef',
    '0x903432f5f5b3d0d548103809ecd1d40215ae12bebb94e39b92e1b7a4cadb43811bbaf6d3e82e2bcabe2bd219a22cde4a',
    '0xb1ef78c32461cf503dca2944b28fbff548acb51192ea781eeda08aca904074886498d320f2be7a55ff2b967f71bef783',
    '0x8066f98ccb56377fdc284cd3a7056790db573ac09db0c80039bd1036bff9299918d442be25d54b8040288b61bf558fbe',
    '0x86743bcf7c579ae7dc2c7d9875e74bb60e5a3d1beb8fd342f66f9a6d73154b0f1232886d9ee2ef7b1b4fe28f3963bc96',
    '0x9598e0bd6099ae9f832840b2382be2cc9aa49b55999d30655d565416caa33b1f3bcd5162c4ee4eddb52489c1f34bb8a6',
    '0xa732ac235b4b92d422ddc04d52a224cd3757038573bda7c4aa5447cc92ff9e8f32c767bad402fb774984c78401e99172',
    '0x83105167c0e5b0ff2be128cf9953305946bedb8a38a4864a8e3d468af8c538714e79f01c9abc340cede50ede6137ec44',
    '0x97f4911623a733d8bb556d061b7ed36e20bb81eae4e0ab62838e56148354a53223dabbf0ed135a253e5380233e6f725c',
    '0xb2749274d214f445f268596f3359754f4460ed6c0e2b452060c8f9a739dd899aa73a8df99173e56612efe5ab6cb08f04',
    '0xb3d923bf75492e934a5865a84d4a3627cdd108c49bf08bd64126302cea8410750e1e4fb57a416f911fb04cf4e16071b2',
    '0xb1c2319875022b266cafc3bfd81d5b1f2c395b31017747690f5d3ba7ba0b7ec3119d74ceec65817087ba4b33c6dc724b',
    '0x91080ea020e2eab0b7f40b7bacd6b0742a214c78c4e38a9200e10ceb7d50b9742c7fbbeabd4293ed556123f398254883',
    '0xb3cfdb0871134ffb5c4bddb5a27167abc82bf27fb20c4d92f864f72d08a02b1cccc31a5ca5e6feef4bf133df59df90ad',
    '0x8805104c695be9080616ae65a6faa912198cd5b416c6bab6a11e9b7edb65ad2c4e1a34f9769dbf07fdcee2cffa4ca597',
    '0xb5aa68ec1e14caa4228d22c10cf3967b655fbae090c845180f55c8b3eb98d662604569016c72e881d964ecfa42a2b38f',
    '0xb744a5619977170ae4ebf8106a8ded6c94ee2d66676245994d9fb1ee48a2142cd5dcdd31eb180670e904fd9a3e8464a3',
    '0x8027dca2dc4d8e9d69ebc0c6bf477de88d1170218facb1e36d5be65b9a83f81b39513ee892eca87ce08955beb2b0a79a',
    '0xacc20d32c31dc2e8e94e2e545e8b7385f3c1b33d83cfbdbd1dfca90a89df5fa6fd427bacfa48f2e749dbea762104c745',
    '0x8a10b4707109376d1b76f90fc1dc76926c42f0cb98d755f99c053f8a56dbe7149446ad8342405ffb0a3fbe8898c2691e',
    '0x99fe24c47a703962d5973a9e2b8a303a70438a3e6481e993516f5c53da30f3c0cd9f58a49a5ce75e17eb6ef97db7d659',
    '0xa59a2742e354b509357a2e8f53f6e719bbfdcb846721705782043d9b832c64c2cfad4e15e8a3aa88131a9b36e347041c',
    '0xb8406d7cffd96c88ea33e38e3e475bbee8197e03cf12206a6cdce29617ee9ca1cb81f4892e894edff0939c43ff2686fb',
    '0x85b59286a9cfa926873a8981beeeaf8b27247043ab0770537941ec7784188a4b5f4b7bf34ac679a9dd7cd443ac5971b2',
    '0x92edec989f373e04cac16e5235ffd0e8a47c5b6097e24c0ee9a123e3ff5361658bc397d0a6df599e770aa4e4c294af28',
    '0xb328ab53a038ba266b7a5be18281759298e3691ce3f1d34b5406809d4a7386e417847befa6784c3dea4f099a98cc1fa7',
    '0xa9e074023d2b1b34ff22391e4b2a7f3ce0ae61830c88131b6d1e134cfcb33e59db8b3e6fb384074a7ca0bc1a770ab423',
    '0x81c53e52a59a68e70de0e7ab4b3a5208c655d6c34b99d5c02dbb479caa4247b689cc9497a9e206a6108e3b0d75e74b00',
    '0x8ad615d69311e7d921a6afe7ef92ffd716237208c04151204f656b39cfda4ca644f47397cddb9dc7174d9a15db27c030',
    '0xb7e72fe7406692551e40e061932d14ffd2d2d6f76efcf57a960399c3140a7c607127eeaf64e089ef47ba2133bce1c572',
    '0x99313c7fddd5527a88d360117ac1e29173c216d54a6801bfc2970b387f27be94e554faff63feb2ff29aa54168b7eb74d',
    '0xb31606718ba578a383033ff1c2a72aa17e2a9ea13de6dbac7be31338a9ebdac63e1e615b990cdf7cb26f6db505867059',
    '0x83c5d4d928dd8c5915cb80ac655975b7a593fd7d542246bdc05e9a0f06e5d9a979d123520b194b7e0a9c67299ab6df41',
    '0x817e625f6b33eecb3fb61c5f829eca1e16910eca85869634d5a6118ec2ab21a44e32a0d06f1fe1235e28b2df46ddb96e',
    '0x99a124137d99b2ea252779111e7101132cd2a48aacdfe67ee2690114d59907f5bfd25765802d40c4d54392980342277f',
    '0xb7bdf97f880d51405ff9e9a9e68ede61dcad774ec62853985e0e71fb1d9b5a8a2f69bb07f212f829f52c7f7f56db4490',
    '0xb6b7309a0a54fed73b61ede9a1ccd13a3495b64b56c2925880b1b584435c0d8e5070a79e108e2cb182e61d67bfd2e9df',
    '0x8cd6db0c0f7f34088f979a74392fa1d7c4160187306a0dac3f2029865f769b21cd4f99cdb6f386e2594b05fb3051e99f',
    '0x93fef558199158961d61aa590a493eae37ce0a8907f0e108802fb1f626088d2c8b23605d67f477fa113aeba7652bc712',
    '0xb07dcd269757910cd7a0e8a185e9dfcaad7824653bf3516084153b636be9ec95874881992102e74428ebaf367c72d908',
    '0xad6c14faff0e51e012131abe9aca7d1b1f418455331a6c4bbfdb6cbf2b710c8508e451e4b6692cbb46135894dd94149c',
    '0x90578c1e23e285c4146dc32109d93dbefe0ce38ae6b6ef823d74d0dd52879641713d2ce5251ef1ce2d52708844de9658',
    '0x88b0dea6e9731348a8ce91359472e6ab1bfd2e6f74e0428a04a1693bb34f6d7db03ba25d801d8a2881b2ad6f7f2e200a',
    '0x8df96121ed873ff68ca6208324bc193015cf38486f4391fe03c49adbabccebacf248676750575a641c0164b8d3da8a8c',
    '0x95cdec46cfbba0c7d2eb6180076b9c5c6d513e40766c2813dfb53606b787977872619b2fccdcbb676cc6890765e7ce63',
    '0xa91476b9af7fb87f4e1487839bf893e2506b604d4f9d95ff92c9683fb0bbcd1aa3257fa757295766659c52f82badc628',
    '0x90aba5a59d20e6a9c57205634da8746d01c27883444140bba06a2b40edd3f5625e843bf775f5b5d1c067653075626abe',
    '0xb2af0934c7ceaa27419c5c8fd794ea54bad708b482048aadafc35f38e894535b4494e16e9beec33336776c9d24197ece',
    '0xb121fc10d6342f8ecdd0269b52bdad1f5d03780b690825b003ed4cf814590b23492b5ff476913753ba0f414d90a2061e',
    '0xa4419c4f5665312484bb4338931e2e9eee56bd9f4be841981bca1a97944efa74e20daa41b4629c256c52287d79b729d9',
    '0xb60caefe7332944072793cf0ecb2f25405bef6b7b1a0f79a5f9f6451136516261d9a341928e3465d6706db6714af2bfb',
    '0xb0fa2f45e99e6dce3c3d04d889dcef9d784f2ddc3220d192070ccaea3c57a538b47aee3cc3efd866a6d06158874e9238',
    '0x877baf5abafbd8f010fca205c6500e44265888aa975417a9adedc454c6e903eec923e936fb9a19a409ffb2f7e7c137d7',
    '0x8b317420ed5fe0dc2a84c905a4297e2ade96476d8100efe6d37e5f68f3d58117083e6bc82e2078b0e2220ea350aadc94',
    '0x9091a56dc67bf8b34d532101089c84ae1f00001f6742b8c0af896a82a5ecfe39d4d78914e1bba52273dba325bc63425c',
    '0xab189e13e5696ab07f5c0701bfeab8ff46c6762b5ef2e800d52079a0963bd8f07f79a899e4ec522f18a0ea059955528c',
    '0xb430ad33eb203e784c090c663c8c14e7b2ed6c4bb94673069153fd10c6ceb2d5d087ba51b882e0ea7d8dd59917e8c1c1',
    '0xa54be70a19efb974211f870a709e9bd89f4ad138c2a251c52a25366b4cf2cca2bced4f3f3e927162dcc169ff5a92a175',
    '0x99702d5ca5a6505ef9236fb490a10b5d25bcb99454a2e3c8e85492938ea6a60b547b761a9a33d9b5f37bd6256dcdac97',
    '0xa7b91a00969bd76b8d7cba15bb0eb9faf543d6454e6bcdb0b2dcb46326828832b795b1c306963b783d70d7196491ec42',
    '0xa845eda8d5a173bc350cb54e34c7ff42ad658ace56b7318ae808b47f4e81057761db1a4f92595cb7c0c1fd61ef9797b0',
    '0xa384735cc1edb32da0bc7bd86a1342228f93bf932e0338e7ae4a784ad54d2e465042afafc462c945cd1b8996b7151979',
    '0xac3e7d1522d3716dc03b8d2b067120f93af87e0ef47f730947ea3915904610b3bab6baa3ca97181dd2293288063aeec6',
    '0xa0be74b56d9ddd474d19b160d672e67b846839d75087ea97031e7a5cf9ea8fe1c72932bfd12d8eb3f207730f6d899c25',
    '0xb4fc806de258bcd1d54052c759199a0ff75063f7cbf8f20b26cd29859cfff0833189fd9d7cdb83ec0e87c27d374937c8',
    '0xa2b103f66aa2abbc5be170bcd1bd9ec8e82433115479f4291cde6a400d1f70e77674f9d5d9b173ed97f0c8a24934ba92',
    '0xa03cbeee359789f508ef15a855df2342cdc55570547e3b8ea7e8749b8218ccb89d12037bab4ce4c96bb0c31c070e152c',
    '0xa04b3acfad935295797aa7dba2ce70cd306defbfe184751e7f4c45a0ca48376c91b8992b70e5e07ccaef79cb151a0466',
    '0x8b0505477bc888f75073abc681386b2ee365e379c46150693c9e14fa8ba9270bcb462c9f80e29f13a2de16c9d90725d7',
    '0xb494c0cf2554a725c19cb1325f40983b1d6c5b3417461e6afa6c301bccbecbda72431753ca0452b5840c999a55e8bafd',
    '0xad631ac89aba1e3b81d44a9f6c4bb9caecf97ee4940db8e6ed591c24be4f127d23efcc382b893cc32bb0b5649c393273',
    '0x8b8774064d1d471fdee1604815923bddd54192d23c1fa872a09ca2b9b9496ada15e45c533129945c845bbd41449fbbd1',
    '0xb35f99a85d43036d3347886d9657d01319c996f74ac3e58e66a503554e41b338e09891e1d4c7528141b057e3bfa9bb02',
    '0x9032035ebc532168f12c16a00f0fd4522cd405ff2634543465ba0cee55e6e17035950ce425fb4ca66819b5719cf62f3f',
    '0xa3235981c095c63d9fce1ce94d0d76ec6515dde9d7a0e190ec945d47131a59086c661699b07c48b733457a4a42072765',
    '0xa7dc3c563dd1ca846fa1a1a13a6d5b5aa108270fc9019db4382a0843d5081843441e1220a409367c4f1476ce4daaf8ec',
    '0x8ce1a8f016c7e61e16485d25dde5490d254bd1e58e9716e2e86340adca39b735ca6d7406fdaf4e87be44f598a8ec7894',
    '0xb8b9c3d438a7ae80d13f244eeab3501432b59146e145403a5c60f7c6a669b0d86537deacd7787207960dcdd33e2b31c8',
    '0x8576d026bf6629b296f43c1b0497595092bdf7cecf194024708132123e327a5f3718eac8dc545b91784821aedf2017ff',
    '0xb0ba5c57b43da994faf6b8ab215a3c60c6fb70b9b6e0f9dd5887f8255d228496f58b0344d97a1861bb894ed83510666b',
    '0x915ca6873996aa0a3bf82a48d6d02aafca300353f17250d4aa9cd21e9f4e9dcfbc1ae10e6eb6da6d259a416338a54ab4',
    '0xb71127ddacf27ec063c51790ae593ee41f42bc07bd6f95c12f62b6e35eb16c81daf5c948a50fdf2b83969e4002e6aca8',
    '0xaeb7d370cca3487fc36cb85ae1ffe4a45d306f64d418f758ee096a7cd4955fcf68362c0020dafb08ba100d5e68f81bfd',
    '0x93209e4f87bcffa08835e40d6a8409064700ee0f09a12cec27a09271ef30dfad8c872ed8ed46ee72b2bdc5126541bb6c',
    '0xb3ad248b55d56e0dc35cc60fecb7716ef5c3d4801092394eeb3da96094d6673ebfd817b5350e53af17443dab004168f9',
    '0xa2fb254cbb8dc17a8dfd255f29d703fb13dbeaa765e33ba5622b7e9c5e85f5cb3637e624f1e787a67aea7a52d108c79a',
    '0x885e21055123f2437e26c37b3dc92a7d1984c56dff528913906da3e8914cd39487b2d4d3f315f5b14d3e3ee13f5e989b',
    '0x9531a330052234d2eeb27bee6fef9903669bf9de511679e1dede7b8958775c50299bd871d5f41b079fcbaf08b335119e',
    '0xb10f0c0e8d102db0f116d4ba87b3a9c1cae3c1af407c210eaefcf3b5421458abfbbf7938ee309552bd59656de2a8e351',
    '0xa68458d4842a1ec086fa1b49f539e7d223eea5fa6cb7dba9857e2a6147b206cd4a066f139bb246e8d1bab15ab07319eb',
    '0x8793d1a1b24e9ee7ae7c31c90c4410d6659e169e1d32c18c8c0ddee583d26b5797c6e5a9218ea8f6e4bb473401c06684',
    '0xa3240235a362e3c21daddd9e1cfdd179fe77fc56cdbb4976e262b60c8dea319b9b66bf7d24b14e04bdee3353ed02bb8a',
    '0x8b7a9ed2e3bb855102478002e7f13ce0543280a9d6b8ae0fa2cf838657f0b43fa7795a31b79805b5fc08fe6ce3d6658b',
    '0xa873d219d74c9e81218ac6bd59cfc3685a6aa44ec962cce6dacc9cf2b27c2f7f958661576120e67d34a76ed96db4eff4',
    '0x8254b6899d3ecae4b2d574a61e1c398db4325f53cff473433501c33b60ba56cda144fde78b3d2aad81462539888eaeff',
    '0x8c2448fb84e225ddac076c2ccb4eb4cd711c155d54239b8e04f1e6f4af8c0d7c3f066307768206f70da1fc5871d8b1af',
    '0x82f2a4d46ad86d18c42514ab0d5f2f9fb113f7c0e8505843af9b36d86eb8bc60ea7681fcad012f419ba6f504a6b64b90',
    '0x827ed8af4c4e741f9bd85c81f96e034a6ebc341ee7a514b6462f31de6cf7a3a10154a57158366f37b5aa1c6c4d490f00',
    '0xb4c474e0df0036935c7031e72c7e7b1297cb83481d4ede99f23114a8b32ee29d18cb9f6211c483c617b7d9cf37a4d896',
    '0xa74bf30955155747e152ad7bb45fdb5decff52611c21167617885e63d312e84762266b24115fa8a630a8ad432b805db7',
    '0x93b6032a4b69d681dc647e1ede686f9f3363313c6fb9e3210f6aa9f21d322000f1a48682e161cd60f296e08dd64d8b95',
    '0x85f8721fc3ed0e30ca047381cd32613d519b12e971cafdb195da8cace310d1ac61c179978b160a82a7372c29b6379067',
    '0x85b0c0fb753c2b29934c39d709835d3e1bf43562363273bc78e9cbc2a18f6b986d1e6b788fc69f339adc3fdef22ac01b',
    '0xb8675e55766cbce96a979f91c31aab07985101b905074cb14698c206665dfc4080004dba1a280ad66873c9e8c7e8467f',
    '0xa79f02b704c2a5cad97fe399b1f99786ba413fafc6f3d9092e14c46f951a475deae5de8a762975e5370923b4134a1c2e',
    '0xa54dcfe2859e31ef86c8c7bc1bb744a2334c1f431bf905b49d4bb39a3e2c25e072e06669e656c60911a34b71d94e607a',
    '0x84632be9b7aad5ff55c7cfd6cf5253ce8f215f79811e2299be107d60514dceb4cc6134b58aa318f232766ec139e6c69d',
    '0xb82915996055b46305cee423c862680b1ca337915f35355bca3ca0c474527c8943407a428349d9e50ced1f674f547ab8',
    '0xb9972f9d2ddc4697ac053f6c39f953ed9b8a620fa787b7d0e0bd0468b4ff2de0391b40328249b6d0c7bd80a3e091663d',
    '0xa1a95e35574a0ab79634a53070651e5998d6841cb9cd442e735001a6ab89396667689c5750b4eb84f6f886efe6fcff30',
    '0xae1f7c6d8d8bf3b44a57c2e9f53b37e5f9b63efd74372e330bd01720198d319b39035adc82adc238c0f331eff30de08d',
    '0xb04910e22a9a9b297345c0653572687b1caf0dd928d8b879fd9b768307ecf410bdf442bdc8fd7c72c35bac3ed7268e54',
    '0xa4bf65e39768ab8c21474e0bb0d44087cb1ace3c156ce092923177ea5779ecff471ae922078415296c16255c333abd3c',
    '0x90da4ea3bae2e497944953c6e3183ee9cf998390140cab6594b00a583cecde10e634cd7e2fd34bc6056681d1ce69ae67',
    '0xb5b482df280c31c38aa1cc0816452819ee6025c18f42a54ec7cf72f8035487f25f7e705a59032487d65b73adcda1853a',
    '0xb3520e472e5206768de36f16c59537ba7a5f548f1e606ef341be57c61c69b36dd3e9b6fe7fe8a36269e9f6b3b6860612',
    '0x8710898bf2943a716120e45223f89f78964a01687dd641607bae4c5decd4a0e7237be539b1697d819290c73c01d6a029',
    '0xa47b201503e8f99d9c9ed436421ba2ad63e9e69272b1420dfb8b01e1dac89fd11cd5772d8ef99b243c446688259369e3',
    '0xa955dab65a0af8ababfd910b2bc315c3fb368caf259e4dee671c71c6f3add7f4c058cde9d08e1ffaffe4d741c9d115c8',
    '0x94fe66fbef86b95b1a2110b4ed6da878fb02358f2b48d351ebcae8efa8eed0c7716adf2eeb6375c6d90c29e7b9ca3b15',
    '0xa808f567d9b451b58147f00d976595bba94c3e8bba6fb2a41ea595087c864dd531db6e8cbc39dc5d2b9c0cc111dc5788',
    '0xb4d73576a705db540ea576ddb17ed11d95ff2ddbd8ba1154bf07676deabbc019a230ba1acae80624136c3b0f9b95ee30',
    '0xadc05eae1133defef956a1436ee978e13965fce854077feb6d6f2bc0799017927db1837917076e10cefb435090c17a20',
    '0x995b6d4b4bd7861095d3648d685db41544513118870a11efd7f9cc65f743703deffa43c9809e58a45d4c9b3af7986bb9',
    '0x83028093be3436a695b396410d79d0204adc1d5ba4c8744462436a840ed947b0f2871da9f493949bcae3d27f38a0772d',
    '0xa09da9b090d8623c77095f074932a87dcc1d8503b9a6a1da6be74b2e1bcc389a28be12d83087116e9addac14328321da',
    '0xb26697026e988d772c9528e8adb2e95c26fe3757a930766fb48e3c0e33f320365838790c9ae3d426b14763a43891f28d',
    '0xa8eeba7abd1295066f2e83682316d688bf7f23a0b64b3394bb3b9a567f0642f09b859bd68418a585d862c54ac5c4d289',
    '0x89580fa433fde9be2792039a548f2555ae904bea9bf65b2bb7e38a60a9b9fe03e07b5ec64399661b15c104279f4f88a6',
    '0xa19d7d0b2386e0490cf5e97f8c29ed1d6dda73be4817c160c1ddb2d89bd292f9dbb3e933733d5afc0da9cc95a842a4d5',
    '0x8729505c542a817d6f5ef5f16898365bb2d358e22664a7b4ebb5281bbb8a7e447e881e5848c56927b4d617533b5eeb86',
    '0xb1cef442d85469c9de1557d524755c2aa7032634fe3ee4cc04cea934c039cad73974a7a2dfa7c26191a266c42ab50698',
    '0xae2a3b4485def751f3da4af720cc65682e4ace95f0758dda4e0b522cc5dd947da92a723ee486339f35849ce2f7e0e14d',
    '0xa9dcd2a450a383af950bbc63b9835ec93e1342c4ab4f7d70992db43a5fbcbdbe9601f138ff77be84dd9967693f008fa5',
    '0xabff5d3432b203ab4cd3a36a1b7e054b170b52785538c7071da6f00af4306a4962ce1858b21fdc00cc70755dcbb31e8d',
    '0x877afeb0290229fa199f82087069467fd97aeaccf58fa1c71ad8a6a8b0151aed0d4b0d00093f1da0d99d83858ff671e3',
    '0x8780af313f74949b51e35899baf75d3c371a31cecaccd40605aa6237ad9f58e923ccadfd429c50ca3c0c0ed5e45094ef',
    '0xa400ca2a121599afcfc6ba8d59759600c81861618afedc71b48b7235d66b1aa145580bc447cb7a46f29fdd3683e216af',
    '0x8d62843368322593e47ae1105e6cda7b82f54f49cd7cd7f9dbe6512dbe9e04f4fcddd99e6219e9259e014b039ee85318',
    '0xa15628a5a5b5f8d5e05341c0b7e0e9855c779fc21532e8e3a4b1b536a2d5c71994238921a77fe4251e0587d444e883b0',
    '0xa30349751dde787daf5cab591c9725b7b2bb242351c619583ceeebd7b0195076f1d9276e0c06f9bec7c81bc1d275e5ce',
    '0x97a2ce12d649b1ea5976131ff7084ff585aea3064afe921ba9143301e1d84e2dbe86f42a29515634b1ac073d2012d3a9',
    '0xb1f226da0f29ccd59ef5dd5189c47a0f1f9c6a76086a864d29af6c2b962567dd13a984e863d65cec6d6aa67f1b02cf6b',
    '0x87fa6005b76abc2b1f3a8e8faaa27b07c96bfed225280c269d8b513570ef05fae8272ca40bba12082777bad842e379e3',
    '0xb5c0f36ceb6ab1b205e58adcd02e7d48803d8836079f35babb9009fd29ca674ed44883a177d6c2eb7f7b717eb3c80d12',
    '0x99cebfec5a78750febd6301aebf08e51a46b2ab117140b0b949e26d65a528ee77814d8dd592b69cc36aac4da373f8dad',
    '0x97a834d630a10c4b08afa918b4c895ea2239afff0743de24fa6e9654e0c36f04e5aa06b3578c5214dd3d999d52e70bed',
    '0x8e0f917a062c5851e94e868c03551818b2f37cb91ff145f421e63e2e6cbafa757581c011273cfffcb3d6a0d4982e81cf',
    '0x93c2d58cebb72a1fce1ccb9c3a6c971f7159b9a257b259feca646e340a9cf7e1c1ba2d95264bd2e5a4aece6d4ade69ea',
    '0x8314edc8a5676bd303d94083bc5573aea7ad6da6a64c56787e3f6932566f676fbd17351a551fa82497407ba6b9b45314',
    '0x88cc6067ef9c96e2394c522c838b32cc853788351a8505bd5c4113dc2e346acda2a010ee34c13a6d3220cda03b44b657',
    '0x94baba80ab3ee8c46fb0e5a4dd0974660ba0ba10e07e4ac136819d4871123d35e38ac86d2c1af119cf69e06f1180ebad',
    '0xaa7e2ef5bf45ac627879efd63ac78c4c96a3fe8d825ac5d8634c45b521ac2b76df0c47a3617942b77dd2dfa4d1724e4d',
    '0xabfe464c0163820f0b4e9bdf2fe74da7bfdf49fb387e7a00d6a96e1df135c4153b0dcc5cbb7e8bd0224f197e308182a1',
    '0xb9f6a958e61dcd2e5d006f7fbe524959ea16331cde2c6cff4b2e46fd617682062b070156510807334eaf20cad944a944',
    '0xaf664e259b5d1ad2cb77a679b09a6b8ddfefbc32922e40e45d57c7d5767819969037494c276a7eb32b08dd4304e9580e',
    '0x956c06d55664dadccc434dde64c31dfba6e2c2783678b722f6740f297b3ef43cfd16b5888aeae399c24fef35ecb5760e',
    '0xb97b2d091b15269566a51d8cb8ef56413a6fdc90f4812b1392a11cb83293603516a77161350996e35a0417fe28315aa5',
    '0x888b8ca332e388a5eb9ce6c96fa45a0d97f950c12ef07d819b407f9829a88da79028f6117c60436781ec6630e945ff9c',
    '0x8f9d4d7472b2d6f3dcac6baf2c3794917f88e5461d061b7f7c02ba5493f332e85579347f4499d7e0f2b434c70816178b',
    '0xb2d431031dd400029cefc8e47136076e0b69e46e1c420901655dee272882df06c4ea420bc0e2bbee1a550686a3689e91',
    '0x86c4574bf5b3dccae10d1cdc81dd7cc732b2e2c455562dd9ec7be186ea66639d8178481ac599d1d7f314a919de2e9dc4',
    '0xa7eb91d77fa58b19a53a815835613fed8bd491241868363e4cd1f378d84d9c00e7eddefec51a7de667113d697a5c8df3',
    '0x8d07e1f522d42cfd4f19da3c2ab00a5375ce9441c6e381a743cd316dd8fbbcd7ea138fe8469e34779994de7396616434',
    '0xb2658ad649dbfe57a70265edbad9c3c43c11c562454f6c9047a1afa15e8405a2f526620125cda4ebf44ee144487da785',
    '0x89187967ec41dda419c06f4b1a23ddbe0262b145410354016af381567eb6719136179bc32f7308d6dcef2d59a3d797bb',
    '0xac38acf5c626f39d7196997bb3d14d7d2630bfaac724deeb6fa7c700e87b6875f11ff34db7fb54814f349adf94335182',
    '0x831e02e683c5f3546fd7ed3b4ae371a2c9c9328245363eea77c8bb5a0266da0440d5079419925fe385a823b2d82d502f',
    '0xb4a0cfc2698b2366abc6adeb8147b386f9810c2b22557c6af30bf181082b148ce3e66247c41a93c40cb380c52dc9cd4c',
    '0x86298a784d75c564dbcc6c2c27de818cbb8bc75c13edc371bd79235cf1ca15d50dd04243ca3b0f9da226b7190132241a',
    '0xb0ab17d41440c4abdbe56d229b26fe226de67f137c8f1c6e7e53386b055a8f81a83521b891de77be9a811a7253f7ccfc',
    '0xb9dc5ce66180c71ca5f57ffc7ca3d249f33bb2ce0d36ecbd93b5fa555d7395db117f78de1544ddc039ee8cc12eb9cdf1',
    '0x8062d02108ad4b35d8c8095155bf8486ab96012c33d9cca2cd5c0a5b20858839316f1f9582d381a309fe8528c8000039',
    '0x95e8316469e357e5936da97b5cc2b26f99c2a2a1ea7858285bac11c35c57a565be18855ce6514b37cd1075e12b4937db',
    '0xa2d64176e34efa6c215d7ae784a9c01946273a049c004b996f3d99838f13a60a9638555abb39fe073c56deafaf9b2633',
    '0xb15407c34fac3e1aa61332913a2b378eda35e46da33e2d6f634f7e0fff58788c76a58e710fe9bce70219a80123e578bc',
    '0xb2a6f8b738ebb0f377c38a80ddb0af13a5b5c9abf33cc882661e2e46b7ab496629612bcaaafe0150e285a0651b64aa74',
    '0xa534ae116af4e46e15d2eec31bfe7a3b03cb0a2e6e16fc6410d94e649f43c759ecb4cdcf8faff6a00445b21eab9c15e2',
    '0xb930cb55519c3a032756679bd37d290bad8c8fb7ce84e7314086c28bfe4bcc831e9b09482a218c2fa5259f331f800d09',
    '0xaecb7e5856707f3d5c07236eecb1e43c88541dee7fd5a7b8973830616c164d29fa177f3a737b79d1a3a95c6709c46dd1',
    '0xb516561130c1d53415674a8e7cc86d765eb6bae5d64a45dce937a0e5cc30a2bd24327620b4c978a5d715d5265ed05cbe',
    '0x94525e493958cab16d9a3d9d238dfab537a581347d7884cbbb715cd7189d9288862d0daa65bb093e45c64d5afaff7cc5',
    '0xb92faf54a4e15a22b0cea30ce48a5f0f023286fae791128b474231bcbe4825f4ea63747e039f108171facbdb9bf3ebb9',
    '0x99d4c556e3f5ef799689616487e26ec568009e1893b4366d2a039ad29cf3be69cb615819a6d02950c1ad858e12104dfa',
    '0xa45a02ef607afb62b3817f9ec755a2ed6a03504370267f2163c28f020f565f67d02f2a99044406d7ca77a69699a97513',
    '0x8dfe4b858340f3324447a1098a365f1a05cfe339378ec617d823f08684e695469d856d22f0e0aba325443aa4d0f73bb6',
    '0x846a60912b2a7e214c24160f708316a234678f6be96001435efe59af0a14ead14a71e577563c5a6cc170b2493c6dbf4b',
    '0xa6768ec021c0bfde4062f410aa4d33ad664a18f34dabb514534b829d6f85ce5188c808aca0878adcb75c6946506403bb',
    '0x8fbcb9bbd1b30c8493a1eaf08c6052ddb07621cfcc2b7096cc434df7e2b9a6247652c7fe1d26640839b56a204b524550',
    '0xb02b44e41d4950c2b108c51f48231e5330f77fe4b5fd6304c79ce908f3e65348a45944cc179e487e93f1bcc93aba13a7',
    '0xb042945f7686973f5508a47b1154e005d5b33b6e36a74c7dcebeda8ef250d1415627252698d57709d020a58c84629d4c',
    '0x897949b3a7c3a7c6fd3d71610712b30c8a9b90bb5cbf3fbf52923c41d631cb9668f5cd5052005399272eafe2736bd9a5',
    '0xab6ef19197ff7bce2a1d524c5f34caf45514dcfb66dc6f02b097678a90b87e9eb2c252f4af760171d5cd299988c98a0c',
    '0x8c176593a396937947c2ff860d7484815bb1afefc9aa97820c7ac9aaa561699365b8df4a2e138792fcf8779bac1430dd',
    '0xa9bdcebdd18575cc603010e5b0587ad4795b56f8084b36f3b8b27780d3648b76791133b7f976c5cd64abd4d342d1bdf1',
    '0x89967e9697e52a661e266b52c1b42355d07826f01cc8bb7f7b22791bcb14100ad11e9d0f73f6035583a8c01915618f90',
    '0x85f2bd6f26482fd2bcfa861b42b06dc68d4a4866fd3ba05a5b58e005cfd96035a022723b446805ffe1f55ca9ad9ed0aa',
  ];

  let pubKeyPoints = pubkeys.map(x => PointG1.fromHex(formatHex(x)).toAffine());
  let pubKeysArray = pubKeyPoints.map(x => [
    bigint_to_array(55, 7, x[0].value),
    bigint_to_array(55, 7, x[1].value),
  ]);

  let pubKeysArrayStr = pubKeysArray.map(pair => [
    pair[0].map(bigInt => bigInt.toString()),
    pair[1].map(bigInt => bigInt.toString()),
  ]);

  let jsonObject = {
    pubkeys: pubKeysArrayStr,
  };

  // // Convert the JSON object to a JSON string
  // let jsonString = JSON.stringify(jsonObject, null, 2); // Pretty print with 2-space indentation

  // // Write JSON string to a file
  // fs.writeFile('poseidonInputs.json', jsonString, 'utf8', err => {
  //   if (err) {
  //     console.error('Error writing file:', err);
  //   } else {
  //     console.log('File has been saved.');
  //   }
  // });
  let poseidon = await buildPoseidonReference();

  let poseidonValFlat: string[] = [];
  for (let i = 0; i < 512; i++) {
    for (let j = 0; j < 7; j++)
      for (let l = 0; l < 2; l++) {
        poseidonValFlat[i * 7 * 2 + j * 2 + l] = pubKeysArray[i][l][j];
      }
  }

  let prev: any = 0;

  const LENGTH = 512 * 2 * 7;
  const NUM_ROUNDS = LENGTH / 16;
  for (let i = 0; i < NUM_ROUNDS; i++) {
    let inputs: any[] = [];
    for (let j = 0; j < 16; j++) {
      inputs.push(poseidonValFlat[i * 16 + j]);
    }
    if (i < NUM_ROUNDS - 1) {
      prev = poseidon(inputs, prev, 1);
    } else {
      prev = poseidon(inputs, prev, 2);
    }
  }

  const res = poseidon.F.e(
    '18983088820287088885850106087039471251611359596827931776044660470697434019038',
  );
  // console.log('res', res);
  // console.log('eq', poseidon.F.eq(res, prev[1]));
  return res;
}

function toLittleEndian(value: bigint): bigint {
  value =
    ((value &
      BigInt(
        '0xFF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00',
      )) >>
      BigInt(8)) |
    ((value &
      BigInt(
        '0x00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF00FF',
      )) <<
      BigInt(8));
  value =
    ((value &
      BigInt(
        '0xFFFF0000FFFF0000FFFF0000FFFF0000FFFF0000FFFF0000FFFF0000FFFF0000',
      )) >>
      BigInt(16)) |
    ((value &
      BigInt(
        '0x0000FFFF0000FFFF0000FFFF0000FFFF0000FFFF0000FFFF0000FFFF0000FFFF',
      )) <<
      BigInt(16));
  value =
    ((value &
      BigInt(
        '0xFFFFFFFF00000000FFFFFFFF00000000FFFFFFFF00000000FFFFFFFF00000000',
      )) >>
      BigInt(32)) |
    ((value &
      BigInt(
        '0x00000000FFFFFFFF00000000FFFFFFFF00000000FFFFFFFF00000000FFFFFFFF',
      )) <<
      BigInt(32));
  value =
    ((value &
      BigInt(
        '0xFFFFFFFFFFFFFFFF0000000000000000FFFFFFFFFFFFFFFF0000000000000000',
      )) >>
      BigInt(64)) |
    ((value &
      BigInt(
        '0x0000000000000000FFFFFFFFFFFFFFFF0000000000000000FFFFFFFFFFFFFFFF',
      )) <<
      BigInt(64));
  value = (value >> BigInt(128)) | (value << BigInt(128));
  return value;
}

function getFirst253Bits(arr: Uint8Array): string {
  if (arr.length !== 32) {
    throw new Error('Input array must be exactly 32 bytes long');
  }

  // Create a new Uint8Array of 32 bytes to hold the first 31 bytes and the modified last byte
  const bytes = new Uint8Array(32);

  // Copy the first 31 bytes
  bytes.set(arr.slice(0, 31));

  // Get the first 5 bits of the 32nd byte (last 3 bits are ignored)
  bytes[31] = arr[31] >> 3;

  // Convert the bytes to a hex string
  const hexString = Array.from(bytes)
    .map(byte => byte.toString(16).padStart(2, '0'))
    .join('');

  // Convert hex string to BigInt
  const bigInt = BigInt('0x' + hexString);

  return bigInt.toString();
}
