import
  std/[os,osproc,strutils],
  confutils,
  config,
  std/json,
  stew/byteutils,
  ../../../../contracts/cosmos/verifier/lib/nim/contract-interactions/helpers

proc execCommand*(): string =
  let conf = ParseDataConf.load()

  case conf.cmd:
    of StartUpCommand.noCommand:
      discard

    of StartUpCommand.initData:
      let vkey = createVerificationKey(conf.verificationKeyPath)
      let hex = hexToByteArray[32]("0xc43d94aaea1342f8e551d9a5e6fe95b7ebb013142acf1e2628ad381e5c713316")
      let init = "{\"vkey\": " & $vkey & ",\"currentHeaderHash\": " &  $hex & "}"
      echo init

    of StartUpCommand.updateData:
      let proof = createProof(conf.proofPath)

      let updateJson = parseFile(conf.updatePath)
      let newOptimisticHeader = hexToByteArray[32](updateJson["attested_header_root"].str)
      let newFinalizedHeader = hexToByteArray[32](updateJson["finalized_header_root"].str)
      let newExecutionStateRoot = hexToByteArray[32](updateJson["finalized_execution_state_root"].str)

      let update= "{\"update\":{\"proof\":" & $proof & ",\"newOptimisticHeader\": " & $newOptimisticHeader & ",\"newFinalizedHeader\": " & $newFinalizedHeader & ",\"newExecutionStateRoot\": " & $newExecutionStateRoot & "}}"

      echo update

    of StartUpCommand.expectedHeaderRootPath:
      echo getExpectedHeaderRoot(conf.expectedHeaderRootPath)

let a = execCommand()

