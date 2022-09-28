import { ArrayifiedContract, StringifiedContract, RawContract } from './types';

export const arrayify = (file: RawContract): ArrayifiedContract => {
  return file.toString().trim().replaceAll('\r', '').split('\n');
};

export const stringify = (a: ArrayifiedContract): StringifiedContract => {
  return a.join('\n');
};

export const numerate = (a: ArrayifiedContract): ArrayifiedContract => {
  return a.map((v, i) =>
    'L'.concat((i + 1).toString().concat(':').padEnd(5, ' ')).concat(v),
  );
};
