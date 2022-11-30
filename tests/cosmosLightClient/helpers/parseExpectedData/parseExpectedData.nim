import
  std/[os,osproc,strutils],
  confutils,
  config,
  ../../../../contracts/cosmos/verifier/lib/nim/helpers

proc execCommand*(): string =
  let conf = ParseExpectedDataConf.load()

  case conf.cmd:
    of StartUpCommand.noCommand:
      discard

    of StartUpCommand.currentHeader:
      echo createCurrentHeader(conf.currentHeaderPath)

    of StartUpCommand.newHeader:
      echo createNewHeader(conf.newHeaderPath)

let a = execCommand()

