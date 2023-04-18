import
  std/[osproc,strutils],
  stew/byteutils,
  helpers,
  confutils,
  config,
  std/json

proc init*(pathVerificationKey, code_id, wallet, node, txflags: string): string =
  let vkey = createVerificationKey(pathVerificationKey)
  let initHeaderHash = hexToByteArray[32]("0xc43d94aaea1342f8e551d9a5e6fe95b7ebb013142acf1e2628ad381e5c713316")

  let INIT = "{\"vkey\": " & $vkey & ",\"current_header_hash\": " &  $initHeaderHash & "}"
  discard execCmdEx("wasmd tx wasm instantiate " & code_id & " '" & INIT & "' --from " & wallet & " --label 'Cosmos Verifier' " & txflags & " -y --no-admin")
  discard execCmdEx("sleep 10")

  var CONTRACT = execCmdEx("wasmd query wasm list-contract-by-code " & code_id  & " " & node & " --output json | jq -r '.contracts[-1]'")[0]
  stripLineEnd(CONTRACT)
  echo CONTRACT
  CONTRACT

proc update*(pathPrf, updatePath, contract, wallet, node, txflags: string): bool =
  let proof = createProof(updatePath)

  let update = parseFile(updatePath)

  let newOptimisticHeader = hexToByteArray[32](update["attested_header_root"].str)
  let newFinalizedHeader = hexToByteArray[32](update["finalized_header_root"].str)
  let newExecutionStateRoot = hexToByteArray[32](update["finalized_execution_state_root"].str)


  let UPDATE= "{\"update\":{\"proof\":" & $proof & ",\"new_optimistic_header_root\": " & $newOptimisticHeader & ",\"new_finalized_header_root\": " & $newFinalizedHeader & ",\"new_execution_state_root\": " & $newExecutionStateRoot & "}}"
  echo "Executing:"
  echo "➤ wasmd tx wasm execute " & contract & " '" & UPDATE & "' --amount 999ustake --from " & wallet & " "  & txflags & " -y "

  echo execCmdEx("wasmd tx wasm execute " & contract & " '" & UPDATE & "' --amount 999ustake --from " & wallet & " "  & txflags & " -y ")

  true

proc query*(contract, node, txflags: string): bool =
  const NAME_QUERY = "{\"header\": {}}"
  let qData =  execCmdEx("wasmd query wasm contract-state smart " & contract & " '" & NAME_QUERY & "' " & node & " --output json")
  echo qData.output
  true

proc execCommand*(): string =
  let conf = CosmosVeryfierConf.load()

  let NODE="--node " & conf.rpc
  let TXFLAG=NODE & " --chain-id " & conf.chainId & " --gas-prices 0.25ustake --gas auto --gas-adjustment 1.3"

  case conf.cmd:
    of StartUpCommand.noCommand:
      discard

    of StartUpCommand.init:
      discard init(conf.vKeyPath, conf.code_id, conf.wallet, NODE, TXFLAG)

    of StartUpCommand.update:
      discard update(conf.proofPath, conf.updatePath, conf.contract, conf.wallet, NODE, TXFLAG)

    of StartUpCommand.query:
      discard query(conf.contract2, Node, TXFLAG)

discard execCommand()

