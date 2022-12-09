import
  std/[os,osproc,strutils],
  confutils,
  config,
  ../../../../contracts/cosmos/verifier/lib/nim/helpers

proc execCommand*(): string =
  let conf = ParseDataConf.load()

  case conf.cmd:
    of StartUpCommand.noCommand:
      discard

    of StartUpCommand.initData:
      let currentHeader = createCurrentHeader(conf.initHeaderPath)
      let vkey = createVerificationKey(conf.verificationKeyPath)
      let init = "{\"vkey\": " & $vkey & ",\"currentHeader\": " &  $currentHeader & "}"
      echo init
      # writeFile("initData.json", init)

    of StartUpCommand.updateData:
      let proof = createProof(conf.proofPath)
      let newHeader = createNewHeader(conf.nextHeaderPath)
      let update= "{\"update\":{\"proof\":" & $proof & ",\"newHeader\": " & $newHeader & "}}"
      echo update
      # writeFile("updateData.json", update)

    of StartUpCommand.currentHeader:
      echo createCurrentHeader(conf.currentHeaderPath)

    of StartUpCommand.newHeader:
      echo createNewHeader(conf.newHeaderPath)

let a = execCommand()

