import
  std/[os, strutils],
  confutils,
  std/json,
  stew/byteutils

import
  config,
  ./helpers

proc execCommand*(): string =
  let conf = ParseDataConf.load()

  case conf.cmd:
    of StartUpCommand.noCommand:
      discard

    of StartUpCommand.initData:
      let vkey = createVerificationKey(conf.verificationKeyPath)
      let hex = hexToByteArray[32](conf.initHeaderRoot)
      let domain = hexToByteArray[32](conf.domain)

      let init = "{\"vkey\": " & $vkey & ",\"current_header_hash\": " &  $hex & ",\"current_slot\": " &  $5609069 & ",\"domain\": " &  $domain & "}"
      echo init

    of StartUpCommand.updateData:
      let proof = createProof(conf.proofPath)
      let updateJson = parseFile(conf.updatePath)

      let newOptimisticHeader = hexToByteArray[32](updateJson["attestedHeaderRoot"].str)
      let newFinalizedHeader = hexToByteArray[32](updateJson["finalizedHeaderRoot"].str)
      let newExecutionStateRoot = hexToByteArray[32](updateJson["finalizedExecutionStateRoot"].str)
      let slot = updateJson["attestedHeaderSlot"]

      let update= "{\"update\":{\"proof\":" & $proof & ",\"new_optimistic_header_root\": " & $newOptimisticHeader & ",\"new_finalized_header_root\": " & $newFinalizedHeader & ",\"new_execution_state_root\": " & $newExecutionStateRoot & ",\"new_slot\": " & $slot & "}}"
      echo update

    of StartUpCommand.updateDataForRelayTest:
      let proofJson = parseFile(conf.proofPathRelay)
      let a = proofJson["pi_a"]
      let b = proofJson["pi_b"]
      let c = proofJson["pi_c"]

      let updateJson = parseFile(conf.updatePathRelay)
      let newOptimisticHeader = updateJson["attestedHeaderRoot"]
      let newFinalizedHeader = updateJson["finalizedHeaderRoot"]
      let newExecutionStateRoot = updateJson["finalizedExecutionStateRoot"]
      let slot = updateJson["attestedHeaderSlot"]

      let update = "{\"attestedHeaderRoot\": " & $newOptimisticHeader & ",\"finalizedHeaderRoot\": " & $newFinalizedHeader & ",\"finalizedExecutionStateRoot\": " & $newExecutionStateRoot &  ",\"a\":" & $a &   ",\"b\":" & $b &  ",\"c\":" & $c & ",\"attestedHeaderSlot\": " & $slot & "}"
      echo update

    of StartUpCommand.expectedHeaderRootPath:
      echo getExpectedHeaderRoot(conf.expectedHeaderRootPath)

    of StartUpCommand.expectedFinalizedRootPath:
      echo getExpectedFinalizedRoot(conf.expectedFinalizedRootPath)

    of StartUpCommand.expectedExecutionStateRoot:
      echo getExpectedExecutionStateRoot(conf.expectedExecutionStateRoot)

    of StartUpCommand.expectedSlot:
      echo getExpectedSlot(conf.expectedSlot)

    of StartUpCommand.updateDataEOS:
      let proof = createProof(conf.proofPathEOS)

      let updateJson = parseFile(conf.updatePathEOS)
      let newOptimisticHeader = hexToByteArray[32](updateJson["attestedHeaderRoot"].str)
      let newFinalizedHeader = hexToByteArray[32](updateJson["finalizedHeaderRoot"].str)
      let newExecutionStateRoot = hexToByteArray[32](updateJson["finalizedExecutionStateRoot"].str)
      let slot = updateJson["attestedHeaderSlot"]

      let update= "'{\"key\":\"dendreth\", \"proof\": \"" & $proof.toHex() & "\" ,\"new_optimistic_header_root\": \"" & $newOptimisticHeader.toHex() & "\" ,\"new_finalized_header_root\": \"" & $newFinalizedHeader.toHex() & "\" ,\"new_execution_state_root\": \"" & $newExecutionStateRoot.toHex()  & "\" ,\"new_slot\": \"" & $slot & "\" } '"
      echo update

    of StartUpCommand.initDataEOS:

      let vkey = createVerificationKey(conf.verificationKeyPathEOS)
      let hex = hexToByteArray[32](conf.initHeaderRootEOS)
      let domain = hexToByteArray[32](conf.domainEOS)

      let init = "\'{\"key\":\"dendreth\", \"verification_key\": \"" & $vkey.toHex() & "\" ,\"current_header_hash\": \"" & $hex.toHex() & "\" ,\"current_slot\": \"" & $5609069 & "\" ,\"domain\": \"" & $domain.toHex() &  "\" }\'"
      echo init

let a = execCommand()
