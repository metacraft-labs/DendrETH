import
  std/[os,osproc,strutils],
  confutils,
  config,
  std/json,
  # ../../../../vendor/nimcrypto/nimcrypto/[sha2, hash, utils],
  stew/byteutils,
  ../../../../contracts/cosmos/verifier/lib/nim/helpers


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
      # writeFile("initData.json", init)

    of StartUpCommand.updateData:
      let proof = createProof(conf.proofPath & "proof" & conf.numberOfUpdate & ".json")
      # let newOptimisticHeader = hexToByteArray[32]("0x51e177b2e6e99ae2b2179f54a471031622713a321d407b56fc3293c0d3d634bb")
      # let newFinalizedHeader = hexToByteArray[32]("0x320129973260d56499e4a85e436ca57775be7b024ad04f7aee97019628d2b1cb")
      # let newExecutionStateRoot = hexToByteArray[32]("0x79a462ed5b52be97b8c887f92c2111b2a4d04cc3fef85ce1e5fcd9bf2e958f7b")
      # let newHeader = createNewHeader(conf.nextHeaderPath)

      let updateJson = parseFile(conf.proofPath & "update" &  conf.numberOfUpdate & ".json")
      let newOptimisticHeader = hexToByteArray[32](updateJson["attested_header_root"].str)
      let newFinalizedHeader = hexToByteArray[32](updateJson["finalized_header_root"].str)
      let newExecutionStateRoot = hexToByteArray[32](updateJson["finalized_execution_state_root"].str)

      let update= "{\"update\":{\"proof\":" & $proof & ",\"newOptimisticHeader\": " & $newOptimisticHeader & ",\"newFinalizedHeader\": " & $newFinalizedHeader & ",\"newExecutionStateRoot\": " & $newExecutionStateRoot & "}}"

      echo update
      # writeFile("updateData.json", update)

    of StartUpCommand.currentHeader:
      echo createCurrentHeader(conf.currentHeaderPath)

    of StartUpCommand.newHeader:
      echo createNewHeader(conf.newHeaderPath)

let a = execCommand()

