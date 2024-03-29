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
    updateDataForCosmosContractClass
    updateDataForEOSContractClass
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

    of updateDataForCosmosContractClass:
      attested_header_root* {.
        desc: "attested_header_root"}: string
      finalized_header_root* {.
        desc: "finalized_header_root"}: string
      finalized_execution_state_root* {.
        desc: "finalized_execution_state_root"}: string
      a* {.
        desc: "proof - point a"}: seq[string]
      b* {.
        desc: "proof - point b"}: seq[string]
      c* {.
        desc: "proof - point c"}: seq[string]
      attested_header_slot* {.
        desc: "attested_header_slot"}: string

    of updateDataForEOSContractClass:
      attested_header_rootEOS* {.
        desc: "attested_header_root"}: string
      finalized_header_rootEOS* {.
        desc: "finalized_header_root"}: string
      finalized_execution_state_rootEOS* {.
        desc: "finalized_execution_state_root"}: string
      aEOS* {.
        desc: "proof - point a"}: seq[string]
      bEOS* {.
        desc: "proof - point b"}: seq[string]
      cEOS* {.
        desc: "proof - point c"}: seq[string]
      attested_header_slotEOS* {.
        desc: "attested_header_slot"}: string

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
