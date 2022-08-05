import bls, { init } from "@chainsafe/bls";
import { hexToBytes } from "../../utils/bls";
import { getInputSignature } from "../../utils";
import { PointG1 } from "@noble/bls12-381";
console.log("OK");
(async () => {
  await init("herumi");
  const result = await getInputSignature("b31ec527b1821c904bcffa5c80447f88efcc43752fe72f73659b8616d5d6e934f36d635455b0c6122ff78566628feabd", "8a61ab5882c991b3fa488c47979901ab4d7853fef18b87130a9e5cbb3522cc10c88d671fd25e3040121d345a4d74c1bf0eb1831274974665fcc5811bebdb95dd36b9778774e5975a29a83988afb9622c7adef05c7eb83c927484eeb077d0d841", "99b25de3a036ba10aca594fbecc34e7ad0f3b6db2b74157f55b0bff7c0663a04");
  console.log(JSON.stringify(result));

})();
