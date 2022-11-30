import
  confutils/defs

type
  StartUpCommand* = enum
    noCommand
    currentHeader
    newHeader

type
  ParseExpectedDataConf* = object
    case cmd* {.
      command
      defaultValue: noCommand }: StartUpCommand

    of noCommand:
      discard

    of currentHeader:
      currentHeaderPath* {.
        desc: "Path to some header"}: string

    of newHeader:
      newHeaderPath* {.
        desc: "Path to some header"}: string
