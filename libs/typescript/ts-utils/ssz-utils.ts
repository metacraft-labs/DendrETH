import { readJson } from './wasm-utils.js';
import { Type } from '@chainsafe/ssz';

export async function jsonToSerializedBase64<T>(
  sszType: Type<T>,
  path: string,
) {
  const jsonContent = await readJson(path);
  const data = sszType.fromJson(jsonContent);
  const serializedData = sszType.serialize(data);
  var b64Data = Buffer.from(serializedData).toString('base64');
  return b64Data;
}
