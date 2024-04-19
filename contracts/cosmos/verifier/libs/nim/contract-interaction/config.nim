import
  confutils/defs

type
  StartUpCommand* = enum
    noCommand
    init
    update
    query

type
  CosmosVeryfierConf* = object

    chainId* {.
      defaultValue: "testing"
      desc: "Chain ID" }: string

    rpc* {.
      defaultValue: "http://localhost:26657"
      desc: "Address of the network" }: string

    wallet* {.
      defaultValue: "fred --keyring-backend test --keyring-dir $HOME/.wasmd_keys"
      desc: "Wallet used for signing all transactions"}:string

    case cmd* {.
      command
      defaultValue: noCommand }: StartUpCommand
    of noCommand:
      discard

    of init:
      vKeyPath* {.
        desc: "Path to the verification key"}: string
      codeId*{.
        desc: "Contract ID"}: string

    of update:
      proofPath* {.
        desc: "Path to the proof"}: string
      updatePath* {.
        desc: "Path to the new header"}: string
      contract* {.
        desc: "Contract Hash"}: string

    of query:
      contract2* {.
        desc: "Contract Hash"}: string
