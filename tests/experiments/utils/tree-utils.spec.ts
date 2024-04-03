import { describe, test, expect } from '@jest/globals';

import {
  isLeaf,
  range,
  iterateLevel,
  iterateTree,
  parentAndNeighbourFromGIndex,
  indexToGIndex,
  gIndexToIndex,
  gIndexToDepth,
} from './tree-utils';
import { fromDepth } from './gindex';

describe('Tree Utils Tests', () => {
  function testIterator<T>(iterable: Iterable<T>, expectedElements: T[]) {
    expect([...iterable]).toEqual(expectedElements);
  }

  describe('iterateTree', () => {
    test('generates correct indices and gIndices for a given depth with no leaf restrictions', () => {
      const iterator = iterateTree({ depth: 3n });
      const expectedIndices = [
        { levelIndex: 0n, gIndex: 8n, level: 3n },
        { levelIndex: 1n, gIndex: 9n, level: 3n },
        { levelIndex: 2n, gIndex: 10n, level: 3n },
        { levelIndex: 3n, gIndex: 11n, level: 3n },
        { levelIndex: 4n, gIndex: 12n, level: 3n },
        { levelIndex: 5n, gIndex: 13n, level: 3n },
        { levelIndex: 6n, gIndex: 14n, level: 3n },
        { levelIndex: 7n, gIndex: 15n, level: 3n },
        { levelIndex: 0n, gIndex: 4n, level: 2n },
        { levelIndex: 1n, gIndex: 5n, level: 2n },
        { levelIndex: 2n, gIndex: 6n, level: 2n },
        { levelIndex: 3n, gIndex: 7n, level: 2n },
        { levelIndex: 0n, gIndex: 2n, level: 1n },
        { levelIndex: 1n, gIndex: 3n, level: 1n },
        { levelIndex: 0n, gIndex: 1n, level: 0n },
      ];

      testIterator(iterator, expectedIndices);
    });

    test('generates correct indices and gIndices for a given depth with leaf restrictions', () => {
      const iterator = iterateTree({ depth: 3n, lastLeafIndex: 5n });
      const expectedIndices = [
        // Do not remove the comments below, they are used to keep track of the expected indices
        { levelIndex: 0n, gIndex: 8n, level: 3n },
        { levelIndex: 1n, gIndex: 9n, level: 3n },
        { levelIndex: 2n, gIndex: 10n, level: 3n },
        { levelIndex: 3n, gIndex: 11n, level: 3n },
        { levelIndex: 4n, gIndex: 12n, level: 3n },
        // { levelIndex: 5n, gIndex: 13n, level: 3n },
        // { levelIndex: 6n, gIndex: 14n, level: 3n },
        // { levelIndex: 7n, gIndex: 15n, level: 3n },
        { levelIndex: 0n, gIndex: 4n, level: 2n },
        { levelIndex: 1n, gIndex: 5n, level: 2n },
        { levelIndex: 2n, gIndex: 6n, level: 2n },
        // { levelIndex: 3n, gIndex: 7n, level: 2n },
        { levelIndex: 0n, gIndex: 2n, level: 1n },
        { levelIndex: 1n, gIndex: 3n, level: 1n },
        { levelIndex: 0n, gIndex: 1n, level: 0n },
      ];

      testIterator(iterator, expectedIndices);
    });
  });

  describe('iterateLevel', () => {
    test('generates correct indices and gIndices for a given level', () => {
      testIterator(iterateLevel(0n), [{ gIndex: 1n, levelIndex: 0n }]);

      testIterator(iterateLevel(1n), [
        { gIndex: 2n, levelIndex: 0n },
        { gIndex: 3n, levelIndex: 1n },
      ]);

      testIterator(iterateLevel(2n), [
        { gIndex: 4n, levelIndex: 0n },
        { gIndex: 5n, levelIndex: 1n },
        { gIndex: 6n, levelIndex: 2n },
        { gIndex: 7n, levelIndex: 3n },
      ]);

      testIterator(iterateLevel(3n), [
        { gIndex: 8n, levelIndex: 0n },
        { gIndex: 9n, levelIndex: 1n },
        { gIndex: 10n, levelIndex: 2n },
        { gIndex: 11n, levelIndex: 3n },
        { gIndex: 12n, levelIndex: 4n },
        { gIndex: 13n, levelIndex: 5n },
        { gIndex: 14n, levelIndex: 6n },
        { gIndex: 15n, levelIndex: 7n },
      ]);

      testIterator(iterateLevel(4n), [
        { gIndex: 16n, levelIndex: 0n },
        { gIndex: 17n, levelIndex: 1n },
        { gIndex: 18n, levelIndex: 2n },
        { gIndex: 19n, levelIndex: 3n },
        { gIndex: 20n, levelIndex: 4n },
        { gIndex: 21n, levelIndex: 5n },
        { gIndex: 22n, levelIndex: 6n },
        { gIndex: 23n, levelIndex: 7n },
        { gIndex: 24n, levelIndex: 8n },
        { gIndex: 25n, levelIndex: 9n },
        { gIndex: 26n, levelIndex: 10n },
        { gIndex: 27n, levelIndex: 11n },
        { gIndex: 28n, levelIndex: 12n },
        { gIndex: 29n, levelIndex: 13n },
        { gIndex: 30n, levelIndex: 14n },
        { gIndex: 31n, levelIndex: 15n },
      ]);
    });

    test('generates correct indices and gIndices for a given level with a finalGIndex', () => {
      const { levelStart, levelEnd } = fromDepth(0n);
      expect(levelStart).toBe(1n);
      expect(levelEnd).toBe(1n);

      testIterator(iterateLevel(0n, 0n), []);

      testIterator(iterateLevel(0n, 1n), [{ gIndex: 1n, levelIndex: 0n }]);

      testIterator(iterateLevel(0n, 2n), [{ gIndex: 1n, levelIndex: 0n }]);

      testIterator(iterateLevel(0n, 3n), [{ gIndex: 1n, levelIndex: 0n }]);

      testIterator(iterateLevel(1n, 0n), []);

      testIterator(iterateLevel(1n, 1n), [{ gIndex: 2n, levelIndex: 0n }]);

      testIterator(iterateLevel(1n, 2n), [
        { gIndex: 2n, levelIndex: 0n },
        { gIndex: 3n, levelIndex: 1n },
      ]);

      testIterator(iterateLevel(1n, 3n), [
        { gIndex: 2n, levelIndex: 0n },
        { gIndex: 3n, levelIndex: 1n },
      ]);

      const depth = 2n;
      const leafNodes = 2n;
      const iterator = iterateLevel(depth, leafNodes);
      const expectedIndices = [
        // Do not remove the comments below, they are used to keep track of the expected indices
        { gIndex: 4n, levelIndex: 0n },
        { gIndex: 5n, levelIndex: 1n },
        // { gIndex: 6n, levelIndex: 2n },
        // { gIndex: 7n, levelIndex: 3n },
      ];

      testIterator(iterator, expectedIndices);
    });
  });

  describe('range', () => {
    {
      const begin = 1n;
      const end = 1n;
      const iterator = range(begin, end);
      const expectedIndices = [];

      testIterator(iterator, expectedIndices);
    }

    test('generates correct indices and gIndices for a given range', () => {
      {
        const begin = 2n;
        const end = 3n;
        const iterator = range(begin, end);
        const expectedIndices = [{ levelIndex: 0n, gIndex: 2n }];

        testIterator(iterator, expectedIndices);
      }
      {
        const begin = 5n;
        const end = 8n;
        const iterator = range(begin, end);
        const expectedIndices = [
          { levelIndex: 0n, gIndex: 5n },
          { levelIndex: 1n, gIndex: 6n },
          { levelIndex: 2n, gIndex: 7n },
        ];

        testIterator(iterator, expectedIndices);
      }
    });
  });

  describe('parentAndNeighbourFromGIndex', () => {
    test('returns correct parent and neighbour for given odd gIndex', () => {
      const gIndex = 5n;
      const expected = { parent: 2n, neighbour: 4n };
      expect(parentAndNeighbourFromGIndex(gIndex)).toEqual(expected);
    });

    test('returns correct parent and neighbour for given even gIndex', () => {
      const gIndex = 4n;
      const expected = { parent: 2n, neighbour: 5n };
      expect(parentAndNeighbourFromGIndex(gIndex)).toEqual(expected);
    });
  });

  describe('indexToGIndex', () => {
    test('returns correct values for the given depth and last leaf index', () => {
      {
        const depth = 0n;
        const index = 0n;
        const expected = 1n;
        expect(indexToGIndex(index, depth)).toEqual(expected);
      }

      {
        const depth = 3n;
        const firstIndex = 0n;
        const firstGIndexExpected = 8n;
        const lastIndex = 7n;
        const lastGIndexExpected = 15n;
        //           depth = 3
        // index:  0 1 2  3  4  5  6  7
        // gIndex: 8 9 10 11 12 13 14 15
        //         ^                  ^
        expect(indexToGIndex(firstIndex, depth)).toEqual(firstGIndexExpected);
        expect(indexToGIndex(lastIndex, depth)).toEqual(lastGIndexExpected);
      }

      {
        const depth = 3n;
        const index = 4n;
        const expected = 12n;
        //           depth = 3
        // index:  0 1 2  3  4  5  6  7
        // gIndex: 8 9 10 11 12 13 14 15
        //                   ^
        expect(indexToGIndex(index, depth)).toEqual(expected);
      }
    });
  });

  describe('gIndexToIndex', () => {
    test('returns correct values for the given depth and last leaf index', () => {
      {
        const depth = 0n;
        const gIndex = 1n;
        const expected = 0n;
        expect(gIndexToIndex(gIndex, depth)).toEqual(expected);
      }

      {
        const depth = 3n;
        const firstGIndex = 8n;
        const firstIndexExpected = 0n;
        const lastGIndex = 15n;
        const lastIndexExpected = 7n;
        //           depth = 3
        // gIndex: 8 9 10 11 12 13 14 15
        // index:  0 1 2  3  4  5  6  7
        //         ^                  ^
        expect(gIndexToIndex(firstGIndex, depth)).toEqual(firstIndexExpected);
        expect(gIndexToIndex(lastGIndex, depth)).toEqual(lastIndexExpected);
      }

      {
        const depth = 3n;
        const gIndex = 12n;
        const expected = 4n;
        //           depth = 3
        // gIndex: 8 9 10 11 12 13 14 15
        // index:  0 1 2  3  4  5  6  7
        //                   ^
        expect(gIndexToIndex(gIndex, depth)).toEqual(expected);
      }
    });
  });

  describe('gIndexToDepth', () => {
    test('returns correct level for the given gIndex', () => {
      {
        const gIndex = 1n;
        const expected = 0n;
        expect(gIndexToDepth(gIndex)).toEqual(expected);
      }
      {
        const gIndex = 8n;
        const expected = 3n;
        expect(gIndexToDepth(gIndex)).toEqual(expected);
      }
      {
        const gIndex = 12n;
        const expected = 3n;
        expect(gIndexToDepth(gIndex)).toEqual(expected);
      }
      {
        const gIndex = 15n;
        const expected = 3n;
        expect(gIndexToDepth(gIndex)).toEqual(expected);
      }
      {
        const gIndex = 16n;
        const expected = 4n;
        expect(gIndexToDepth(gIndex)).toEqual(expected);
      }
    });
  });

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
});
