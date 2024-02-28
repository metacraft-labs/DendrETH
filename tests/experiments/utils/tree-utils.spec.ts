import {
  childrenFromGIndex,
  isLeaf,
  fromDepth,
  log2,
  range,
  iterateLevel,
  iterateTree,
  parentAndNeighbourFromGIndex,
  indexToGIndex,
  gIndexToIndex,
} from './tree-utils';

describe('Tree Utils Tests', () => {
  function testIterator(iterator, expectedIndices) {
    for (const expected of expectedIndices) {
      const result = iterator.next();
      console.log(result);
      expect(result.value).toEqual(expected);
    }
    expect(iterator.next().done).toBe(true);
  }

  describe('iterateTree', () => {
    test('generates correct indices and gIndices for a given depth with no leaf restrictions', () => {
      const depth = 4n;
      const iterator = iterateTree(depth);
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
      const depth = 4n;
      const maxLeafIndex = 5n;
      const iterator = iterateTree(depth, maxLeafIndex);
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

      testIterator(iterator, expectedIndices);
    });
  });

  describe('iterateLevel', () => {
    test('generates correct indices and gIndices for a given level', () => {
      const level = 3n;
      const iterator = iterateLevel(level);
      const expectedIndices = [
        { indexOnThisLevel: 1n, gIndex: 4n },
        { indexOnThisLevel: 2n, gIndex: 5n },
        { indexOnThisLevel: 3n, gIndex: 6n },
        { indexOnThisLevel: 4n, gIndex: 7n },
      ];

      testIterator(iterator, expectedIndices);
    });

    test('generates correct indices and gIndices for a given level with a finalGIndex', () => {
      const level = 3n;
      const finalGIndex = 2n;
      const iterator = iterateLevel(level, finalGIndex);
      const expectedIndices = [
        // Do not remove the comments below, they are used to keep track of the expected indices
        { indexOnThisLevel: 1n, gIndex: 4n },
        { indexOnThisLevel: 2n, gIndex: 5n },
        // { indexOnThisLevel: 3n, gIndex: 6n },
        // { indexOnThisLevel: 4n, gIndex: 7n },
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
        { indexOnThisLevel: 1n, gIndex: 5n },
        { indexOnThisLevel: 2n, gIndex: 6n },
        { indexOnThisLevel: 3n, gIndex: 7n },
        { indexOnThisLevel: 4n, gIndex: 8n },
      ];

      testIterator(iterator, expectedIndices);
    });
  });

  describe('childrenFromGIndex', () => {
    test('returns correct leftChild and rightChild for given gIndex', () => {
      const gIndex = 5n;
      const expected = { leftChild: 10n, rightChild: 11n };
      expect(childrenFromGIndex(gIndex)).toEqual(expected);
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
      const depth = 4n;
      const lastIndex = 5n;
      const expected = 12n;

      expect(indexToGIndex(lastIndex, depth)).toEqual(expected);
    });
  });

  describe('gIndexToIndex', () => {
    test('returns correct values for the given depth and last leaf index', () => {
      const depth = 4n;
      const gIndex = 12n;
      const expected = 5n;

      expect(gIndexToIndex(gIndex, depth)).toEqual(expected);
    });
  });

  describe('isLeaf', () => {
    test('returns true if gIndex is a leaf at the given depth', () => {
      const gIndex = 5n;
      const depth = 3n;
      expect(isLeaf(gIndex, depth)).toBe(true);
    });

    test('returns false if gIndex is not part of the tree with this depth', () => {
      const gIndex = 8n;
      const depth = 3n;
      expect(isLeaf(gIndex, depth)).toBe(false);
    });

    test('returns false if gIndex is inner node', () => {
      const gIndex = 2n;
      const depth = 3n;
      expect(isLeaf(gIndex, depth)).toBe(false);
    });
  });

  describe('fromDepth', () => {
    test('returns correct values for the given depth', () => {
      const depth = 3n;
      const expected = {
        beg: 1n,
        end: 7n,
        levelBeg: 4n,
        levelEnd: 7n,
        elementCount: 4n,
      };
      expect(fromDepth(depth)).toEqual(expected);
    });
  });

  describe('log2', () => {
    test('returns correct log2 value for the given number', () => {
      const x = 8n;
      expect(log2(x)).toBe(3n);
    });
  });
});
