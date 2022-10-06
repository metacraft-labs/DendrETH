import { PADDING } from './constants';
import { ArrayifiedContract, StringifiedContract, RawContract } from './types';

export const arrayify = (f: RawContract): ArrayifiedContract => {
  return f.toString().trim().replaceAll('\r', '').split('\n');
};

export const stringify = (a: ArrayifiedContract): StringifiedContract => {
  return a.join('\n');
};

// TODO: remove these
export const numerate = (a: ArrayifiedContract): ArrayifiedContract => {
  return a.map((v, i) =>
    'L'.concat((i + 1).toString().concat(':').padEnd(PADDING, ' ')).concat(v),
  );
};

export const denumerate = (a: ArrayifiedContract): ArrayifiedContract => {
  return a.map(v => v.slice(PADDING + 1));
};
