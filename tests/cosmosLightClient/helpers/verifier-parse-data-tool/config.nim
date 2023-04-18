import
  confutils/defs

type
  StartUpCommand* = enum
    noCommand
    initData
    updateData
    expectedHeaderRootPath
    expectedFinalizedRootPath
    expectedExecutionStateRoot

type
  ParseDataConf* = object
    case cmd* {.
      command
      defaultValue: noCommand }: StartUpCommand

    of noCommand:
      discard

    of initData:
      initHeaderPath* {.
        desc: "Path to some header"}: string
      verificationKeyPath* {.
        desc: "Path to the verification key"}: string

    of updateData:
      proofPath* {.
        desc: "Path to some header"}: string
      updatePath* {.
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
