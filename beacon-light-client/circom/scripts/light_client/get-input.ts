import { writeFileSync } from 'fs';
import path from 'path';
import { getFilesInDir } from '../../../../libs/typescript/ts-utils/data';
import { ZHEAJIANG_TESNET } from '../../../solidity/test/utils/constants';
import { getProofInput } from './get_ligth_client_input';

(async () => {
  const UPDATES = getFilesInDir(path.join(__dirname, 'updates'));

  let prevUpdate = JSON.parse(UPDATES[0].toString());

  for (let update of UPDATES.slice(1, 2)) {
    writeFileSync(
      path.join(__dirname, 'input.json'),
      JSON.stringify(
        await getProofInput(
          prevUpdate,
          JSON.parse(update.toString()),
          ZHEAJIANG_TESNET,
        ),
      ),
    );
  }
})();
