import { expand_message_xmd, stringToBytes, htfDefaults } from "./bls";

(async () => {
    let msg = new Uint8Array([1, 2, 3]);
    const DST = stringToBytes(htfDefaults.DST);

    let result = await expand_message_xmd(msg, DST, 3);

    console.log(result);
})();