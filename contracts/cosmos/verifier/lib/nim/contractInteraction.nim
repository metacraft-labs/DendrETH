import
  std/[os,osproc,strutils],
  helpers,
  confutils,
  config

proc init*(pathVerificationKey, pathCurrentHeader, code_id, wallet, node, txflags: string): string =
  let vkey = createVerificationKey(pathVerificationKey)
  let currentHeader = createCurrentHeader(pathCurrentHeader)

  let INIT = "{\"vkey\": " & $vkey & ",\"currentHeader\": " &  $currentHeader & "}"
  discard execCmdEx("wasmd tx wasm instantiate " & code_id & " '" & INIT & "' --from " & wallet & " --label 'Cosmos Verifier' " & txflags & " -y --no-admin")
  discard execCmdEx("sleep 10")

  var CONTRACT = execCmdEx("wasmd query wasm list-contract-by-code " & code_id  & " " & node & " --output json | jq -r '.contracts[-1]'")[0]
  stripLineEnd(CONTRACT)
  echo CONTRACT
  CONTRACT

proc update*(pathPrf, pathNewHeader, contract, wallet, node, txflags: string): bool =
  let proof = createProof(pathPrf)
  let newHeader = createNewHeader(pathNewHeader)

  let UPDATE= "{\"update\":{\"proof\":" & $proof & ",\"newHeader\": " & $newHeader & "}}"
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
      discard init(conf.vKeyPath, conf.currentHeaderPath, conf.code_id, conf.wallet, NODE, TXFLAG)

    of StartUpCommand.update:
      discard update(conf.proofPath, conf.newHeaderPath, conf.contract, conf.wallet, NODE, TXFLAG)

    of StartUpCommand.query:
      discard query(conf.contract2, Node, TXFLAG)

let a = execCommand()

