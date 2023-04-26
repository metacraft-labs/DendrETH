import
  confutils/defs

type
  StartUpCommand* = enum
    noCommand
    initData
    updateData
    updateDataForRelayTest
    expectedHeaderRootPath
    expectedFinalizedRootPath
    expectedExecutionStateRoot
    updateDataForCosmosClass

type
  ParseDataConf* = object
    case cmd* {.
      command
      defaultValue: noCommand }: StartUpCommand

    of noCommand:
      discard

    of initData:
      initHeaderRoot* {.
        desc: "Root of the header to init with"}: string
      verificationKeyPath* {.
        desc: "Path to the verification key"}: string

    of updateData:
      proofPath* {.
        desc: "Path to some header"}: string
      updatePath* {.
        desc: "updatePath"}: string

    of updateDataForRelayTest:
      proofPathRelay* {.
        desc: "Path to some header"}: string
      updatePathRelay* {.
        desc: "updatePath"}: string

    of expectedHeaderRootPath:
      expectedHeaderRootPath* {.
        desc: "Path to some header"}: string

    of expectedFinalizedRootPath:
      expectedFinalizedRootPath* {.
        desc: "Path to some header"}: string

    of expectedExecutionStateRoot:
      expectedExecutionStateRoot* {.
        desc: "Path to some header"}: string

    of updateDataForCosmosClass:
      attested_header_root* {.
        desc: ""}: string
      finalized_header_root* {.
        desc: ""}: string
      finalized_execution_state_root* {.
        desc: ""}: string
      a* {.
        desc: ""}: seq[string]
      b* {.
        desc: ""}: seq[string]
      c* {.
        desc: ""}: seq[string]
