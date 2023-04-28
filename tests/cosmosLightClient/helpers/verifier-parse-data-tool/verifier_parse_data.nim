import
  std/[os,osproc,strutils],
  confutils,
  config,
  std/json,
  stew/byteutils,
  ../../../../contracts/cosmos/verifier/lib/nim/contract_interactions/helpers,
  bncurve/group_operations


proc execCommand*(): string =
  let conf = ParseDataConf.load()

  case conf.cmd:
    of StartUpCommand.noCommand:
      discard

    of StartUpCommand.initData:
      let vkey = createVerificationKey(conf.verificationKeyPath)
      let hex = hexToByteArray[32](conf.initHeaderRoot)
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

    of StartUpCommand.updateDataForRelayTest:

      let proofJson = parseFile(conf.proofPathRelay)
      let a = proofJson["pi_a"]
      let b = proofJson["pi_b"]
      let c = proofJson["pi_c"]

      let updateJson = parseFile(conf.updatePathRelay)
      let newOptimisticHeader = updateJson["attested_header_root"]
      let newFinalizedHeader = updateJson["finalized_header_root"]
      let newExecutionStateRoot = updateJson["finalized_execution_state_root"]

      let update = "{\"attested_header_root\": " & $newOptimisticHeader & ",\"finalized_header_root\": " & $newFinalizedHeader & ",\"finalized_execution_state_root\": " & $newExecutionStateRoot &  ",\"a\":" & $a &   ",\"b\":" & $b &  ",\"c\":" & $c & "}"

      echo update

    of StartUpCommand.expectedHeaderRootPath:
      echo getExpectedHeaderRoot(conf.expectedHeaderRootPath)

    of StartUpCommand.expectedFinalizedRootPath:
      echo getExpectedFinalizedRoot(conf.expectedFinalizedRootPath)

    of StartUpCommand.expectedExecutionStateRoot:
      echo getExpectedExecutionStateRoot(conf.expectedExecutionStateRoot)

    of StartUpCommand.updateDataForCosmosContractClass:
      var parsedB: seq[seq[string]]

      parsedB.add(@[conf.b[0], conf.b[1]])
      parsedB.add(@[conf.b[2], conf.b[3]])
      parsedB.add(@[conf.b[4], conf.b[5]])

      let newOptimisticHeader = hexToByteArray[32](conf.attested_header_root)
      let newFinalizedHeader = hexToByteArray[32](conf.finalized_header_root)
      let newExecutionStateRoot = hexToByteArray[32](conf.finalized_execution_state_root)
      let a = Point[G1](x: FQ.fromString(conf.a[0]), y: FQ.fromString(conf.a[1]), z: FQ.fromString("1"))
      let b = Point[G2](x: FQ2(c0: FQ.fromString(parsedB[0][0]),  c1: FQ.fromString(parsedB[0][1])), y: FQ2(c0: FQ.fromString(parsedB[1][0]), c1: FQ.fromString(parsedB[1][1])), z: FQ2(c0: FQ.fromString("1"), c1: FQ.fromString("0")))
      let c = Point[G1](x: FQ.fromString(conf.c[0]), y: FQ.fromString(conf.c[1]), z: FQ.fromString("1"))

      let prf = Proof(a:a, b:b, c:c)
      let proof = cast[var array[sizeof(Proof),byte]](prf.unsafeAddr)
      let update = "{\"update\":{\"proof\":" & $proof & ",\"new_optimistic_header_root\": " & $newOptimisticHeader & ",\"new_finalized_header_root\": " & $newFinalizedHeader & ",\"new_execution_state_root\": " & $newExecutionStateRoot & "}}"
      echo update


let a = execCommand()

