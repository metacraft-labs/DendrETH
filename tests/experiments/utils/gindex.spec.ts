import { describe, test, expect } from '@jest/globals';

import { fromDepth, fromGIndex, log2 } from './gindex';
import { stringify } from './common-utils';

describe('GIndex Tests', () => {
  describe('log2', () => {
    test('log2(0) throws error', () => {
      expect(() => log2(0n)).toThrowError('log2: x must be greater than 0');
    });

    test('log2(1) == 0', () => {
      expect(log2(1n)).toBe(0n);
    });

    test('log2(2) == 1', () => {
      expect(log2(2n)).toBe(1n);
    });

    test('log2(3) == 1', () => {
      expect(log2(3n)).toBe(1n);
    });

    test('log2(4) == 2', () => {
      expect(log2(4n)).toBe(2n);
    });

    test('log2(8) == 3', () => {
      expect(log2(8n)).toBe(3n);
    });

    // test with 15
    test('log2(15) == 3', () => {
      expect(log2(15n)).toBe(3n);
    });

    test('log2(16) == 4', () => {
      expect(log2(16n)).toBe(4n);
    });

    test('log2(32) == 5', () => {
      expect(log2(32n)).toBe(5n);
    });

    test('log2(64) == 6', () => {
      expect(log2(64n)).toBe(6n);
    });

    test('log2(128) == 7', () => {
      expect(log2(128n)).toBe(7n);
    });

    test('log2(173) == 7', () => {
      expect(log2(173n)).toBe(7n);
    });

    test('log2(256) == 8', () => {
      expect(log2(256n)).toBe(8n);
    });

    test('log2(512) == 9', () => {
      expect(log2(512n)).toBe(9n);
    });

    test('log2(1024) == 10', () => {
      expect(log2(1024n)).toBe(10n);
    });
  });

  describe('Depth', () => {
    test('depth == 0', () => {
      expect(fromDepth(0n)).toEqual(
        expect.objectContaining({
          depth: 0n,
          first: 1n,
          last: 1n,
          levelStart: 1n,
          levelEnd: 1n,
          elementCount: 1n,
        }),
      );
    });

    test('depth == 1', () => {
      expect(fromDepth(1n)).toEqual(
        expect.objectContaining({
          first: 1n,
          last: 3n,
          levelStart: 2n,
          levelEnd: 3n,
          elementCount: 3n,
        }),
      );
    });

    test('depth == 2', () => {
      expect(fromDepth(2n)).toEqual(
        expect.objectContaining({
          first: 1n,
          last: 7n,
          levelStart: 4n,
          levelEnd: 7n,
          elementCount: 7n,
        }),
      );
    });

    test('depth == 3', () => {
      expect(fromDepth(3n)).toEqual(
        expect.objectContaining({
          first: 1n,
          last: 15n,
          levelStart: 8n,
          levelEnd: 15n,
          elementCount: 15n,
        }),
      );
    });

    test('depth == 4', () => {
      expect(fromDepth(4n)).toEqual(
        expect.objectContaining({
          first: 1n,
          last: 31n,
          levelStart: 16n,
          levelEnd: 31n,
          elementCount: 31n,
        }),
      );
    });

    test('depth == 5', () => {
      expect(fromDepth(5n)).toEqual(
        expect.objectContaining({
          first: 1n,
          last: 63n,
          levelStart: 32n,
          levelEnd: 63n,
          elementCount: 63n,
        }),
      );
    });

    test('depth == 6', () => {
      expect(fromDepth(6n)).toEqual(
        expect.objectContaining({
          first: 1n,
          last: 127n,
          levelStart: 64n,
          levelEnd: 127n,
          elementCount: 127n,
        }),
      );
    });

    test('depth == 7', () => {
      expect(fromDepth(7n)).toEqual(
        expect.objectContaining({
          first: 1n,
          last: 255n,
          levelStart: 128n,
          levelEnd: 255n,
          elementCount: 255n,
        }),
      );
    });

    test('depth == 8', () => {
      expect(fromDepth(8n)).toEqual(
        expect.objectContaining({
          first: 1n,
          last: 511n,
          levelStart: 256n,
          levelEnd: 511n,
          elementCount: 511n,
        }),
      );
    });

    test('depth == 9', () => {
      expect(fromDepth(9n)).toEqual(
        expect.objectContaining({
          first: 1n,
          last: 1023n,
          levelStart: 512n,
          levelEnd: 1023n,
          elementCount: 1023n,
        }),
      );
    });

    test('depth == 10', () => {
      expect(fromDepth(10n)).toEqual(
        expect.objectContaining({
          first: 1n,
          last: 2047n,
          levelStart: 1024n,
          levelEnd: 2047n,
          elementCount: 2047n,
        }),
      );
    });
  });

  describe('GIndex', () => {
    test('gIndex == 1', () => {
      expect(fromGIndex(1n)).toEqual(
        expect.objectContaining({
          gIndex: 1n,
          depth: 0n,
          levelStart: 1n,
          levelIndex: 0n,
          left: 2n,
          right: 3n,
          parent: 0n,
        }),
      );
    });

    test('gIndex == 2', () => {
      expect(fromGIndex(2n)).toEqual(
        expect.objectContaining({
          gIndex: 2n,
          depth: 1n,
          levelStart: 2n,
          levelIndex: 0n,
          left: 4n,
          right: 5n,
          parent: 1n,
        }),
      );
    });

    test('gIndex == 3', () => {
      expect(fromGIndex(3n)).toEqual(
        expect.objectContaining({
          gIndex: 3n,
          depth: 1n,
          levelStart: 2n,
          levelIndex: 1n,
          left: 6n,
          right: 7n,
          parent: 1n,
        }),
      );
    });

    test('gIndex == 4', () => {
      expect(fromGIndex(4n)).toEqual(
        expect.objectContaining({
          gIndex: 4n,
          depth: 2n,
          levelStart: 4n,
          levelIndex: 0n,
          left: 8n,
          right: 9n,
          parent: 2n,
        }),
      );
    });

    test('gIndex == 5', () => {
      expect(fromGIndex(5n)).toEqual(
        expect.objectContaining({
          gIndex: 5n,
          depth: 2n,
          levelStart: 4n,
          levelIndex: 1n,
          left: 10n,
          right: 11n,
          parent: 2n,
        }),
      );
    });

    test('gIndex == 6', () => {
      expect(fromGIndex(6n)).toEqual(
        expect.objectContaining({
          gIndex: 6n,
          depth: 2n,
          levelStart: 4n,
          levelIndex: 2n,
          left: 12n,
          right: 13n,
          parent: 3n,
        }),
      );
    });

    test('gIndex == 7', () => {
      expect(fromGIndex(7n)).toEqual(
        expect.objectContaining({
          gIndex: 7n,
          depth: 2n,
          levelStart: 4n,
          levelIndex: 3n,
          left: 14n,
          right: 15n,
          parent: 3n,
        }),
      );
    });

    test('gIndex == 8', () => {
      expect(fromGIndex(8n)).toEqual(
        expect.objectContaining({
          gIndex: 8n,
          depth: 3n,
          levelStart: 8n,
          levelIndex: 0n,
          left: 16n,
          right: 17n,
          parent: 4n,
        }),
      );
    });

    test('gIndex == 9', () => {
      expect(fromGIndex(9n)).toEqual(
        expect.objectContaining({
          gIndex: 9n,
          depth: 3n,
          levelStart: 8n,
          levelIndex: 1n,
          left: 18n,
          right: 19n,
          parent: 4n,
        }),
      );
    });

    test('gIndex == 10', () => {
      expect(fromGIndex(10n)).toEqual(
        expect.objectContaining({
          gIndex: 10n,
          depth: 3n,
          levelStart: 8n,
          levelIndex: 2n,
          left: 20n,
          right: 21n,
          parent: 5n,
        }),
      );
    });

    test('gIndex == 11', () => {
      expect(fromGIndex(11n)).toEqual(
        expect.objectContaining({
          gIndex: 11n,
          depth: 3n,
          levelStart: 8n,
          levelIndex: 3n,
          left: 22n,
          right: 23n,
          parent: 5n,
        }),
      );
    });

    test('gIndex == 12', () => {
      expect(fromGIndex(12n)).toEqual(
        expect.objectContaining({
          gIndex: 12n,
          depth: 3n,
          levelStart: 8n,
          levelIndex: 4n,
          left: 24n,
          right: 25n,
          parent: 6n,
        }),
      );
    });

    test('gIndex == 13', () => {
      expect(fromGIndex(13n)).toEqual(
        expect.objectContaining({
          gIndex: 13n,
          depth: 3n,
          levelStart: 8n,
          levelIndex: 5n,
          left: 26n,
          right: 27n,
          parent: 6n,
        }),
      );
    });

    test('gIndex == 14', () => {
      expect(fromGIndex(14n)).toEqual(
        expect.objectContaining({
          gIndex: 14n,
          depth: 3n,
          levelStart: 8n,
          levelIndex: 6n,
          left: 28n,
          right: 29n,
          parent: 7n,
        }),
      );
    });

    test('gIndex == 15', () => {
      expect(fromGIndex(15n)).toEqual(
        expect.objectContaining({
          gIndex: 15n,
          depth: 3n,
          levelStart: 8n,
          levelIndex: 7n,
          left: 30n,
          right: 31n,
          parent: 7n,
        }),
      );
    });

    test('gIndex == 16', () => {
      expect(fromGIndex(16n)).toEqual(
        expect.objectContaining({
          gIndex: 16n,
          depth: 4n,
          levelStart: 16n,
          levelIndex: 0n,
          left: 32n,
          right: 33n,
          parent: 8n,
        }),
      );
    });

    test('gIndex == 1022', () => {
      expect(fromGIndex(1022n)).toEqual(
        expect.objectContaining({
          gIndex: 1022n,
          depth: 9n,
          levelStart: 512n,
          levelIndex: 510n,
          left: 2044n,
          right: 2045n,
          parent: 511n,
        }),
      );
    });

    test('gIndex == 1023', () => {
      expect(fromGIndex(1023n)).toEqual(
        expect.objectContaining({
          gIndex: 1023n,
          depth: 9n,
          levelStart: 512n,
          levelIndex: 511n,
          left: 2046n,
          right: 2047n,
          parent: 511n,
        }),
      );
    });

    test('gIndex == 1024', () => {
      expect(fromGIndex(1024n)).toEqual(
        expect.objectContaining({
          gIndex: 1024n,
          depth: 10n,
          levelStart: 1024n,
          levelIndex: 0n,
          left: 2048n,
          right: 2049n,
          parent: 512n,
        }),
      );
    });
  });

  describe('log2', () => {
    test('returns correct log2 value for the given number', () => {
      expect(log2(1n)).toBe(0n);
      expect(log2(2n)).toBe(1n);
      expect(log2(4n)).toBe(2n);
      expect(log2(8n)).toBe(3n);
      expect(log2(16n)).toBe(4n);
      expect(log2(32n)).toBe(5n);
      expect(log2(64n)).toBe(6n);
      expect(log2(128n)).toBe(7n);
      expect(log2(256n)).toBe(8n);
      expect(log2(512n)).toBe(9n);
      expect(log2(1024n)).toBe(10n);
    });
  });
});
