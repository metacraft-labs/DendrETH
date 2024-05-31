import { Redis } from '@dendreth/relay/implementations/redis';
import { getCommitmentMapperProof } from '../../../utils/common_utils';

(async () => {
  let redis = new Redis('localhost', 6379);

  console.log(
    (await getCommitmentMapperProof(BigInt(5120408), 65536n, 'poseidon', redis))
      .map(
        x =>
          `builder.constant_hash(HashOut {elements:[${x
            .map(i => `GoldilocksField(${i})`)
            .join(',')}]})`,
      )
      .join(','),
  );
})();
