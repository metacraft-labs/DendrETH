import type { JestConfigWithTsJest } from 'ts-jest';

const config: JestConfigWithTsJest = {
  preset: 'ts-jest/presets/default-esm',
  testMatch: ['**/tests/**/*.ts'],
  modulePaths: ['<rootDir>/libs/typescript'],

  moduleNameMapper: {
    '^(\\.{1,2}/.*)\\.js$': '$1',
  },
  modulePathIgnorePatterns: ['vendor/', 'node_modules/', 'tests/helpers'],
};

export default config;
