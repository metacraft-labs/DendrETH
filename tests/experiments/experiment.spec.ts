import {
  createArrayFromRange,
  childLeafsExists,
  checkContent,
} from './experiment';

describe('To the moon', () => {
  const dataDir = 'tests/experiments/test';
  it('createArrayFromRange', () => {
    const result = createArrayFromRange(1, 5);
    expect(result).toEqual([1, 2, 3, 4, 5]);
  });

  it('checkContent', async () => {
    const result = await checkContent(
      `${dataDir}/helloworld.txt`,
      'helloworld',
    );
    expect(result).toEqual(true);
  });

  it('childLeafsExists', async () => {
    const result = await childLeafsExists(dataDir, 1, 1);
    expect(result).toEqual(true);
  });
});
