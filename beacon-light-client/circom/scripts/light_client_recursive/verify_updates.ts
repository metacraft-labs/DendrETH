import { groth16 } from 'snarkjs';
import { readFileSync } from 'fs';

(async () => {
  for (let i = 291; i <= 416; i++) {
    const verificationKey = JSON.parse(
      readFileSync('scripts/light_client_recursive/vkey.json', 'utf8'),
    );
    const pub = JSON.parse(
      readFileSync(
        `../../vendor/eth2-light-client-updates/mainnet/recursive-proofs/public${i}.json`,
        'utf8',
      ),
    );
    const proof = JSON.parse(
      readFileSync(
        `../../vendor/eth2-light-client-updates/mainnet/recursive-proofs/proof${i}.json`,
        'utf8',
      ),
    );

    const isValid = await groth16.verify(verificationKey, pub, proof);

    if (isValid) {
      console.log(`Verified recursive proof for period: ${i}`);
    } else {
      console.log(`Invalid proof`, '\x1b[31m');
      process.exit(1);
    }
  }

  process.exit(0);
})();
