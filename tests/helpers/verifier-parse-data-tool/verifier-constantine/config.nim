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
    expectedSlot
    updateDataEOS
    initDataEOS

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
      domain* {.
        desc: "Domain to init with"}: string
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

    of expectedSlot:
      expectedSlot* {.
        desc: "Path to some header"}: string

    of updateDataEOS:
      proofPathEOS* {.
        desc: "Path to some header"}: string
      updatePathEOS* {.
        desc: "updatePath"}: string

    of initDataEOS:
      initHeaderRootEOS* {.
        desc: "Root of the header to init with"}: string
      domainEOS* {.
        desc: "Domain to init with"}: string
      verificationKeyPathEOS* {.
        desc: "Path to the verification key"}: string
