import
  confutils/defs

type
  StartUpCommand* = enum
    noCommand
    initData
    updateData
    currentHeader
    newHeader

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
      numberOfUpdate* {.
        desc: "numberOfUpdate"}: string

    of currentHeader:
      currentHeaderPath* {.
        desc: "Path to some header"}: string

    of newHeader:
      newHeaderPath* {.
        desc: "Path to some header"}: string
