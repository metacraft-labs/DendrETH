import { describe, test, expect } from '@jest/globals';

import {
  // childrenFromGIndex,
  isLeaf,
  // fromDepth,
  range,
  iterateLevel,
  iterateTree,
  // parentAndNeighbourFromGIndex,
  // indexToGIndex,
  // gIndexToIndex,
  // gIndexToLevel,
  // take,
} from './tree-utils';
import { fromDepth, log2 } from './gindex';

describe('Tree Utils Tests', () => {
  function testIterator<T>(iterator: Generator<T>, expectedIndices: T[]) {
    // console.log([...iterator]);
    expect([...iterator]).toEqual(expectedIndices);
  }

  describe('iterateTree', () => {
    test('generates correct indices and gIndices for a given depth with no leaf restrictions', () => {
      const iterator = iterateTree({ depth: 4n });
      const expectedIndices = [
        { indexOnThisLevel: 1n, gIndex: 8n, level: 4n },
        { indexOnThisLevel: 2n, gIndex: 9n, level: 4n },
        { indexOnThisLevel: 3n, gIndex: 10n, level: 4n },
        { indexOnThisLevel: 4n, gIndex: 11n, level: 4n },
        { indexOnThisLevel: 5n, gIndex: 12n, level: 4n },
        { indexOnThisLevel: 6n, gIndex: 13n, level: 4n },
        { indexOnThisLevel: 7n, gIndex: 14n, level: 4n },
        { indexOnThisLevel: 8n, gIndex: 15n, level: 4n },
        { indexOnThisLevel: 1n, gIndex: 4n, level: 3n },
        { indexOnThisLevel: 2n, gIndex: 5n, level: 3n },
        { indexOnThisLevel: 3n, gIndex: 6n, level: 3n },
        { indexOnThisLevel: 4n, gIndex: 7n, level: 3n },
        { indexOnThisLevel: 1n, gIndex: 2n, level: 2n },
        { indexOnThisLevel: 2n, gIndex: 3n, level: 2n },
        { indexOnThisLevel: 1n, gIndex: 1n, level: 1n },
      ];

      testIterator(iterator, expectedIndices);
    });

    test('generates correct indices and gIndices for a given depth with leaf restrictions', () => {
      const iterator = iterateTree({ depth: 4n, lastLeafIndex: 5n });
      const expectedIndices = [
        // Do not remove the comments below, they are used to keep track of the expected indices
        { indexOnThisLevel: 1n, gIndex: 8n, level: 4n },
        { indexOnThisLevel: 2n, gIndex: 9n, level: 4n },
        { indexOnThisLevel: 3n, gIndex: 10n, level: 4n },
        { indexOnThisLevel: 4n, gIndex: 11n, level: 4n },
        { indexOnThisLevel: 5n, gIndex: 12n, level: 4n },
        // { indexOnThisLevel: 6n, gIndex: 13n, level: 4n },
        // { indexOnThisLevel: 7n, gIndex: 14n, level: 4n },
        // { indexOnThisLevel: 8n, gIndex: 15n, level: 4n },
        { indexOnThisLevel: 1n, gIndex: 4n, level: 3n },
        { indexOnThisLevel: 2n, gIndex: 5n, level: 3n },
        { indexOnThisLevel: 3n, gIndex: 6n, level: 3n },
        // { indexOnThisLevel: 4n, gIndex: 7n, level: 3n },
        { indexOnThisLevel: 1n, gIndex: 2n, level: 2n },
        { indexOnThisLevel: 2n, gIndex: 3n, level: 2n },
        { indexOnThisLevel: 1n, gIndex: 1n, level: 1n },
      ];

      // console.log([...iterator]);

      testIterator(iterator, expectedIndices);
    });
  });

  describe('iterateLevel', () => {
    test('xxxxxxx', () => {
      console.log(...iterateLevel(0n));
      testIterator(iterateLevel(0n), [{ gIndex: 1n, indexOnThisLevel: 0n }]);

      //   testIterator(iterateLevel(1n), [
      //     { gIndex: 2n, indexOnThisLevel: 0n },
      //     { gIndex: 3n, indexOnThisLevel: 1n },
      //   ]);

      //   testIterator(iterateLevel(2n), [
      //     { gIndex: 4n, indexOnThisLevel: 0n },
      //     { gIndex: 5n, indexOnThisLevel: 1n },
      //     { gIndex: 6n, indexOnThisLevel: 2n },
      //     { gIndex: 7n, indexOnThisLevel: 3n },
      //   ]);

      //   testIterator(iterateLevel(3n), [
      //     { gIndex: 8n, indexOnThisLevel: 0n },
      //     { gIndex: 9n, indexOnThisLevel: 1n },
      //     { gIndex: 10n, indexOnThisLevel: 2n },
      //     { gIndex: 11n, indexOnThisLevel: 3n },
      //     { gIndex: 12n, indexOnThisLevel: 4n },
      //     { gIndex: 13n, indexOnThisLevel: 5n },
      //     { gIndex: 14n, indexOnThisLevel: 6n },
      //     { gIndex: 15n, indexOnThisLevel: 7n },
      //   ]);

      //   testIterator(iterateLevel(4n), [
      //     { gIndex: 16n, indexOnThisLevel: 0n },
      //     { gIndex: 17n, indexOnThisLevel: 1n },
      //     { gIndex: 18n, indexOnThisLevel: 2n },
      //     { gIndex: 19n, indexOnThisLevel: 3n },
      //     { gIndex: 20n, indexOnThisLevel: 4n },
      //     { gIndex: 21n, indexOnThisLevel: 5n },
      //     { gIndex: 22n, indexOnThisLevel: 6n },
      //     { gIndex: 23n, indexOnThisLevel: 7n },
      //     { gIndex: 24n, indexOnThisLevel: 8n },
      //     { gIndex: 25n, indexOnThisLevel: 9n },
      //     { gIndex: 26n, indexOnThisLevel: 10n },
      //     { gIndex: 27n, indexOnThisLevel: 11n },
      //     { gIndex: 28n, indexOnThisLevel: 12n },
      //     { gIndex: 29n, indexOnThisLevel: 13n },
      //     { gIndex: 30n, indexOnThisLevel: 14n },
      //     { gIndex: 31n, indexOnThisLevel: 15n },
      //   ]);
    });

    test('generates correct indices and gIndices for a given level with a finalGIndex', () => {
      const { levelStart, levelEnd } = fromDepth(0n);
      expect(levelStart).toBe(1n);
      expect(levelEnd).toBe(1n);
      testIterator(iterateLevel(0n, 0n), []);

      testIterator(iterateLevel(0n, 1n), [
        { gIndex: 1n, indexOnThisLevel: 0n },
      ]);

      testIterator(iterateLevel(0n, 2n), [
        { gIndex: 1n, indexOnThisLevel: 0n },
      ]);

      testIterator(iterateLevel(0n, 3n), [
        { gIndex: 1n, indexOnThisLevel: 0n },
      ]);

      testIterator(iterateLevel(1n, 0n), []);

      testIterator(iterateLevel(1n, 1n), [
        { gIndex: 2n, indexOnThisLevel: 0n },
      ]);

      testIterator(iterateLevel(1n, 2n), [
        { gIndex: 2n, indexOnThisLevel: 0n },
        { gIndex: 3n, indexOnThisLevel: 1n },
      ]);

      const depth = 2n;
      const leafNodes = 2n;
      const iterator = iterateLevel(depth, leafNodes);
      const expectedIndices = [
        // Do not remove the comments below, they are used to keep track of the expected indices
        { gIndex: 4n, indexOnThisLevel: 0n },
        { gIndex: 5n, indexOnThisLevel: 1n },
        // { gIndex: 6n, indexOnThisLevel: 2n },
        // { gIndex: 7n, indexOnThisLevel: 3n },
      ];

      testIterator(iterator, expectedIndices);
    });
  });

  describe('range', () => {
    test('generates correct indices and gIndices for a given range', () => {
      const begin = 5n;
      const end = 8n;
      const iterator = range(begin, end);
      const expectedIndices = [
        { indexOnThisLevel: 0n, gIndex: 5n },
        { indexOnThisLevel: 1n, gIndex: 6n },
        { indexOnThisLevel: 2n, gIndex: 7n },
      ];

      testIterator(iterator, expectedIndices);
    });
  });

  // describe('childrenFromGIndex', () => {
  //   test('returns correct leftChild and rightChild for given gIndex', () => {
  //     const gIndex = 5n;
  //     const expected = { leftChild: 10n, rightChild: 11n };
  //     expect(childrenFromGIndex(gIndex)).toEqual(expected);
  //   });
  // });

  // describe('parentAndNeighbourFromGIndex', () => {
  //   test('returns correct parent and neighbour for given odd gIndex', () => {
  //     const gIndex = 5n;
  //     const expected = { parent: 2n, neighbour: 4n };
  //     expect(parentAndNeighbourFromGIndex(gIndex)).toEqual(expected);
  //   });

  //   test('returns correct parent and neighbour for given even gIndex', () => {
  //     const gIndex = 4n;
  //     const expected = { parent: 2n, neighbour: 5n };
  //     expect(parentAndNeighbourFromGIndex(gIndex)).toEqual(expected);
  //   });
  // });

  // describe('indexToGIndex', () => {
  //   test('returns correct values for the given depth and last leaf index', () => {
  //     const depth = 4n;
  //     const lastIndex = 5n;
  //     const expected = 12n;

  //     expect(indexToGIndex(lastIndex, depth)).toEqual(expected);
  //   });
  // });

  // describe('gIndexToIndex', () => {
  //   test('returns correct values for the given depth and last leaf index', () => {
  //     const depth = 4n;
  //     const gIndex = 12n;
  //     const expected = 5n;

  //     expect(gIndexToIndex(gIndex, depth)).toEqual(expected);
  //   });
  // });

  // describe('gIndexToLevel', () => {
  //   test('returns correct level for the given gIndex', () => {
  //     {
  //       const gIndex = 1n;
  //       const expected = 0n;
  //       expect(gIndexToLevel(gIndex)).toEqual(expected);
  //     }
  //     {
  //       const gIndex = 8n;
  //       const expected = 3n;
  //       expect(gIndexToLevel(gIndex)).toEqual(expected);
  //     }
  //     {
  //       const gIndex = 12n;
  //       const expected = 3n;
  //       expect(gIndexToLevel(gIndex)).toEqual(expected);
  //     }
  //     {
  //       const gIndex = 15n;
  //       const expected = 3n;
  //       expect(gIndexToLevel(gIndex)).toEqual(expected);
  //     }
  //     {
  //       const gIndex = 16n;
  //       const expected = 4n;
  //       expect(gIndexToLevel(gIndex)).toEqual(expected);
  //     }
  //   });
  // });

  describe('isLeaf', () => {
    test('returns true if gIndex is a leaf at the given depth', () => {
      const gIndex = 9n;
      const depth = 3n;
      expect(isLeaf(gIndex, depth)).toBe(true);
    });

    test('returns false if gIndex is not a leaf at the given depth', () => {
      const gIndex = 17n;
      const depth = 3n;
      expect(isLeaf(gIndex, depth)).toBe(false);
    });

    test('returns false if gIndex is inner node', () => {
      const gIndex = 2n;
      const depth = 3n;
      expect(isLeaf(gIndex, depth)).toBe(false);
    });
  });

  // describe('fromDepth', () => {
  //   test('returns correct values for the given depth', () => {
  //     const depth = 3n;
  //     const expected = {
  //       beg: 1n,
  //       end: 7n,
  //       levelBeg: 4n,
  //       levelEnd: 7n,
  //       elementCount: 4n,
  //     };
  //     expect(fromDepth(depth)).toEqual(expected);
  //   });
  // });
});
