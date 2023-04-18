import
  std/[os,osproc,strutils],
  confutils,
  config,
  std/json,
  stew/byteutils,
  ../../../../contracts/cosmos/verifier/lib/nim/contract_interactions/helpers

proc execCommand*(): string =
  let conf = ParseDataConf.load()

  case conf.cmd:
    of StartUpCommand.noCommand:
      discard

    of StartUpCommand.initData:
      let vkey = createVerificationKey(conf.verificationKeyPath)
      let hex = hexToByteArray[32]("0xc43d94aaea1342f8e551d9a5e6fe95b7ebb013142acf1e2628ad381e5c713316")
      let init = "{\"vkey\": " & $vkey & ",\"current_header_hash\": " &  $hex & "}"
      echo init

    of StartUpCommand.updateData:
      let proof = createProof(conf.proofPath)

      let updateJson = parseFile(conf.updatePath)
      let newOptimisticHeader = hexToByteArray[32](updateJson["attested_header_root"].str)
      let newFinalizedHeader = hexToByteArray[32](updateJson["finalized_header_root"].str)
      let newExecutionStateRoot = hexToByteArray[32](updateJson["finalized_execution_state_root"].str)

      let update= "{\"update\":{\"proof\":" & $proof & ",\"new_optimistic_header_root\": " & $newOptimisticHeader & ",\"new_finalized_header_root\": " & $newFinalizedHeader & ",\"new_execution_state_root\": " & $newExecutionStateRoot & "}}"

      echo update

    of StartUpCommand.expectedHeaderRootPath:
      echo getExpectedHeaderRoot(conf.expectedHeaderRootPath)

    of StartUpCommand.expectedFinalizedRootPath:
      echo getExpectedFinalizedRoot(conf.expectedFinalizedRootPath)

    of StartUpCommand.expectedExecutionStateRoot:
      echo getExpectedExecutionStateRoot(conf.expectedExecutionStateRoot)


let a = execCommand()

