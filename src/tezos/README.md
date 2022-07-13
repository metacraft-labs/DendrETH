# Information

Add/Edit your contract files a `./src`

### Directory structure

```shell
<project-directory>
│
├── build                       # Artifacts (compilations and tests)
│   │
│   ├── compilation
│   │   │
│   │   ├── Template.contract   # Contract compilation folder
│   │   └── ...
│   │
│   └── test
│       │
│       ├── Template.contract   # Contract testing folder
│       └── ...
│
├── src                         # Location for contract files (*.contract.ts)
|   |
│   ├── SubModule.ts            # Sub module example (it is imported into Template.contract.ts)
│   └── Template.contract.ts    # Template contract
│
├── templates                   # Various contract templates templates
|   |
│   ├── FA1_2.contract.ts       # FA1.2 template
│   └── FA2.contract.ts         # FA template
|
├── .eslintrc.js                # eslint configuration
├── .prettierrc.js              # prettier configuration
├── package.json
└── tsconfig.json               # It should not be modified.
```

### Running compilations

The command below will iterate over all files in the `src` directory with postfix `*.contract.ts` and run all compilation targets.

```shell
npm run compile
```

### Running tests

The command below will iterate over all files in the `src` directory with postfix `*.contract.ts` and run all test targets.

```shell
npm run test
```

### Deploy contracts

By default, the originator will use a faucet account.
But you can provide your own private key by providing the argument `--private-key`

```shell
npm run originate -- --code build/compilation/<...>_contract.tz --storage build/compilation/<...>_storage.tz --rpc https://mainnet.smartpy.io --private-key <edsk...>
```
