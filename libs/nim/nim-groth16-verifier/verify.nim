when defined(emcc):
  {.pragma: wasmPragma, cdecl, exportc, dynlib, codegenDecl: "__attribute__((used)) $# $#$#".}
else:
  {.pragma: wasmPragma, cdecl, exportc, dynlib.}
import
  bncurve/groups  # std/json

export groups

type
  IC* = array[5, Point[G1]]

  VerificationKey* = object
    alpha*: Point[G1]
    beta*, gamma*, delta*: Point[G2]
    ic*: IC

  Proof* = object
    a*, c*: Point[G1]
    b*: Point[G2]

  Header* = object
    head*: Fr
    tail*: Fr

  Input* = object
    data*: Point[G1]


# get new header and new proof
# create input with current header, new header and IC of VerificationKey
# check if VK+Proof+input = correct
# save new header as current header
#proc prepareHeaders*(currentHeader:HeaderHash,newHeader:HeaderHash,ic:IC)

# proc createVerificationKey2*(path: string): VerificationKey =
#   let vk = 5

#   let vkAlpha1 = Point[G1](x: FQ.fromString(vk["vk_alpha_1"][0].str), y: FQ.fromString(vk["vk_alpha_1"][1].str), z: FQ.fromString("1"))
#   let vkBeta2 = Point[G2](x: FQ2(c0: FQ.fromString(vk["vk_beta_2"][0][0].str),  c1: FQ.fromString(vk["vk_beta_2"][0][1].str)), y: FQ2(c0: FQ.fromString(vk["vk_beta_2"][1][0].str), c1: FQ.fromString(vk["vk_beta_2"][1][1].str)), z: FQ2(c0: FQ.fromString("1"), c1: FQ.fromString("0")))
#   let vkGamma2 = Point[G2](x: FQ2(c0: FQ.fromString(vk["vk_gamma_2"][0][0].str),  c1: FQ.fromString(vk["vk_gamma_2"][0][1].str)), y: FQ2(c0: FQ.fromString(vk["vk_gamma_2"][1][0].str), c1: FQ.fromString(vk["vk_gamma_2"][1][1].str)), z: FQ2(c0: FQ.fromString("1"), c1: FQ.fromString("0")))
#   let vkDelta2 = Point[G2](x: FQ2(c0: FQ.fromString(vk["vk_delta_2"][0][0].str),  c1: FQ.fromString(vk["vk_delta_2"][0][1].str)), y: FQ2(c0: FQ.fromString(vk["vk_delta_2"][1][0].str), c1: FQ.fromString(vk["vk_delta_2"][1][1].str)), z: FQ2(c0: FQ.fromString("1"), c1: FQ.fromString("0")))

#   var icSeq: IC
#   for el in vk["IC"]:
#     let ic = Point[G1](x: FQ.fromString(el[0].str), y: FQ.fromString(el[1].str), z: FQ.fromString("1"))
#     icSeq.add(ic)

#   VerificationKey(alpha:vkAlpha1, beta:vkBeta2, gamma:vkGamma2, delta:vkDelta2, ic:icSeq)

type myDumbKey = object
  protocol: string
  curve: string
  vk_alpha_1: array[3, string]
  vk_beta_2:  array[3, array[2, string]]
  vk_gamma_2:  array[3, array[2, string]]
  vk_delta_2:  array[3, array[2, string]]
  ic:  array[5, array[3, string]]

type myDumbProof = object
  pi_a: array[3, string]
  pi_b: array[3, array[2, string]]
  pi_c: array[3, string]

type myDumbHeader = array[2, string]

type myDumbInput = object
  ic:array[5, array[3, string]]
  oldHeader: array[2, string]
  newHeader: array[2, string]


type
  ThreeStringAddress = array[3, string]

proc testVerify*(vk:int, prf:int, input:int): int {.wasmPragma.} =
  vk - prf - input

proc getVerificationKeySize*(): uint32 {.wasmPragma.} =
  sizeof(VerificationKey).uint32
proc getProofSize*(): uint32 {.wasmPragma.} =
  sizeof(Proof).uint32
proc getHeaderSize*(): uint32 {.wasmPragma.} =
  sizeof(Header).uint32
proc getInputSize*(): uint32 {.wasmPragma.} =
  sizeof(Input).uint32


proc createVerificationKeyWithString*():ref VerificationKey {.wasmPragma.} =
  var vk: myDumbKey = myDumbKey(
    protocol: "groth16",
    curve: "bn128",
    vk_alpha_1:["20491192805390485299153009773594534940189261866228447918068658471970481763042", "9383485363053290200918347156157836566562967994039712273449902621266178545958", "1"],

    vk_beta_2:[["6375614351688725206403948262868962793625744043794305715222011528459656738731","4252822878758300859123897981450591353533073413197771768651442665752259397132"],
    ["10505242626370262277552901082094356697409835680220590971873171140371331206856","21847035105528745403288232691147584728191162732299865338377159692350059136679"],["1","0"]],

    vk_gamma_2:[["10857046999023057135944570762232829481370756359578518086990519993285655852781","11559732032986387107991004021392285783925812861821192530917403151452391805634"],
    ["8495653923123431417604973247489272438418190587263600148770280649306958101930","4082367875863433681332203403145435568316851327593401208105741076214120093531"],["1","0"]],

    vk_delta_2:[["10857046999023057135944570762232829481370756359578518086990519993285655852781","11559732032986387107991004021392285783925812861821192530917403151452391805634"],
    ["8495653923123431417604973247489272438418190587263600148770280649306958101930","4082367875863433681332203403145435568316851327593401208105741076214120093531"],["1","0"]],

    ic:[["12341254398012831539511529514141920531233310925559640868133205893740926624749","1898346778999733876099328904356803438781336221250567894820659652669409421709","1"],
    ["20581758020956979671791380396676180914933879489933044097984142919872524264433","9909799947047067906035029346433967103598628542141844188201546463877414532390","1"],
    ["9676508711308354042906200636781077532751603523420398102911817991603787384838","3283726592693011886356839062982995108864203785773230930373125221081605006490","1"],
    ["13173001357742665986032466031029094851321843920842980071903261453747149717823","20227736002115979502147501163867970082744770988953245358764176832339037911954","1"],
    ["12503202302840927457518240189095194534947514058111893826856048640111724282474","10324849082459144926597406829440562859648647791510582416705714907864823023456","1"]]
  )

  let vkAlpha1 = Point[G1](x: FQ.fromString(vk.vk_alpha_1[0]), y: FQ.fromString(vk.vk_alpha_1[1]), z: FQ.fromString("1"))
  let vkBeta2 = Point[G2](x: FQ2(c0: FQ.fromString(vk.vk_beta_2[0][0]),  c1: FQ.fromString(vk.vk_beta_2[0][1])), y: FQ2(c0: FQ.fromString(vk.vk_beta_2[1][0]), c1: FQ.fromString(vk.vk_beta_2[1][1])), z: FQ2(c0: FQ.fromString("1"), c1: FQ.fromString("0")))
  let vkGamma2 = Point[G2](x: FQ2(c0: FQ.fromString(vk.vk_gamma_2[0][0]),  c1: FQ.fromString(vk.vk_gamma_2[0][1])), y: FQ2(c0: FQ.fromString(vk.vk_gamma_2[1][0]), c1: FQ.fromString(vk.vk_gamma_2[1][1])), z: FQ2(c0: FQ.fromString("1"), c1: FQ.fromString("0")))
  let vkDelta2 = Point[G2](x: FQ2(c0: FQ.fromString(vk.vk_delta_2[0][0]),  c1: FQ.fromString(vk.vk_delta_2[0][1])), y: FQ2(c0: FQ.fromString(vk.vk_delta_2[1][0]), c1: FQ.fromString(vk.vk_delta_2[1][1])), z: FQ2(c0: FQ.fromString("1"), c1: FQ.fromString("0")))

  # var icSeq: IC
  # for el in vk.ic:
  #   let ic = Point[G1](x: FQ.fromString(el[0]), y: FQ.fromString(el[1]), z: FQ.fromString("1"))
  #   icSeq.add(ic)
  var isNotSeq: IC
  for i in 0 .. 4:
    isNotSeq[i] = Point[G1](x: FQ.fromString(vk.ic[i][0]), y: FQ.fromString(vk.ic[i][1]), z: FQ.fromString("1"))


  var keyRef: ref VerificationKey = new VerificationKey
  keyRef.alpha = vkAlpha1
  keyRef.beta = vkBeta2
  keyRef.gamma = vkGamma2
  keyRef.delta = vkDelta2
  keyRef.ic = isNotSeq

  keyRef

proc createProofWithString*(): ref Proof {.wasmPragma.} =
  var proof: myDumbProof = myDumbProof(
  pi_a: ["21123854216953852112536838882481450220498567105940466891840060611781017961528","2098881355538917379090526162958661279125964878419429578543258171869884965366","1"],
  pi_b: [["7306208154393410634846368016070873672969892231814336504289660981281312995280","7600740749950828256586349629860082368206135680276869866284170831205905401844"],
  ["7700657863140842767649251784164958877369431509761612236238859537878274222407","2370973894489812409234438323997614430145127794426485312402498367819060569815"],["1","0"]],
  pi_c:["18203076996553642627496142104359992567986316147417149883557214145940311740993","21693271516513352492458464924120426920468161664319691679281004422356956390755","1"],)

  let a = Point[G1](x: FQ.fromString(proof.pi_a[0]), y: FQ.fromString(proof.pi_a[1]), z: FQ.fromString("1"))
  let b = Point[G2](x: FQ2(c0: FQ.fromString(proof.pi_b[0][0]),  c1: FQ.fromString(proof.pi_b[0][1])), y: FQ2(c0: FQ.fromString(proof.pi_b[1][0]), c1: FQ.fromString(proof.pi_b[1][1])), z: FQ2(c0: FQ.fromString("1"), c1: FQ.fromString("0")))
  let c = Point[G1](x: FQ.fromString(proof.pi_c[0]), y: FQ.fromString(proof.pi_c[1]), z: FQ.fromString("1"))

  var proofRef: ref Proof = new Proof
  proofRef.a = a
  proofRef.b = b
  proofRef.c = c
  proofRef

proc createOldHeaderWithString*(): ref Header {.wasmPragma.} =
  var public: myDumbHeader = myDumbHeader(["5966029082507805980254291345114545245067072315222408966008558171151621124246","4"])
  let oldheader = Header(head: Fr.fromString(public[0]),tail:Fr.fromString(public[1]))
  var oldHeaderRef: ref Header = new Header
  oldHeaderRef.head = oldHeader.head
  oldHeaderRef.tail = oldHeader.tail
  oldHeaderRef
proc createNewHeaderWithString*(): ref Header {.wasmPragma.} =
  var public: myDumbHeader = myDumbHeader(["12857343771181087157409557648182655546684462713036905539892384468792366321123","6"])
  let newheader = Header(head: Fr.fromString(public[0]),tail:Fr.fromString(public[1]))
  var newHeaderRef: ref Header = new Header
  newHeaderRef.head = newHeader.head
  newHeaderRef.tail = newHeader.tail
  newHeaderRef
proc createInputWithString*(): ref Input {.wasmPragma.} =
  var input: myDumbInput = myDumbInput(
  ic:[["12341254398012831539511529514141920531233310925559640868133205893740926624749","1898346778999733876099328904356803438781336221250567894820659652669409421709","1"],
  ["20581758020956979671791380396676180914933879489933044097984142919872524264433","9909799947047067906035029346433967103598628542141844188201546463877414532390","1"],
  ["9676508711308354042906200636781077532751603523420398102911817991603787384838","3283726592693011886356839062982995108864203785773230930373125221081605006490","1"],
  ["13173001357742665986032466031029094851321843920842980071903261453747149717823","20227736002115979502147501163867970082744770988953245358764176832339037911954","1"],
  ["12503202302840927457518240189095194534947514058111893826856048640111724282474","10324849082459144926597406829440562859648647791510582416705714907864823023456","1"]],
  oldHeader:["5966029082507805980254291345114545245067072315222408966008558171151621124246","4"],
  newHeader:["12857343771181087157409557648182655546684462713036905539892384468792366321123","6"]
  )
  let oldheader1 = Header(head: Fr.fromString(input.oldHeader[0]),tail:Fr.fromString(input.oldHeader[1]))
  let newheader1 = Header(head: Fr.fromString(input.newHeader[0]),tail:Fr.fromString(input.newHeader[1]))
  var preparedInputs = Input(data:Point[G1](x: FQ.fromString(input.ic[0][0]), y: FQ.fromString(input.ic[0][1]), z: FQ.fromString("1")))
  preparedInputs.data = preparedInputs.data + (Point[G1](x: FQ.fromString(input.ic[1][0]), y: FQ.fromString(input.ic[1][1]), z: FQ.fromString("1")) * oldHeader1.head)
  preparedInputs.data = preparedInputs.data + (Point[G1](x: FQ.fromString(input.ic[2][0]), y: FQ.fromString(input.ic[2][1]), z: FQ.fromString("1")) * oldHeader1.tail)
  preparedInputs.data = preparedInputs.data + (Point[G1](x: FQ.fromString(input.ic[3][0]), y: FQ.fromString(input.ic[3][1]), z: FQ.fromString("1")) * newHeader1.head)
  preparedInputs.data = preparedInputs.data + (Point[G1](x: FQ.fromString(input.ic[4][0]), y: FQ.fromString(input.ic[4][1]), z: FQ.fromString("1")) * newHeader1.tail)
  var preparedInputsRef: ref Input = new Input
  preparedInputsRef.data = preparedInputs.data
  preparedInputsRef

proc makePairsAndVerifyWithPointers*(vk: ref VerificationKey, prf: ref Proof, oldHeader: ref Header, newHeader: ref Header): bool {.wasmPragma.} =
#   var vk1 = createVerificationKeyWithString()
#   var prf1 = createProofWithString()
#   var preparedInputs1 = createInputWithString()
  # var result = 0
  var preparedInputs = Input(data:vk[].ic[0])
  preparedInputs.data = preparedInputs.data + (vk[].ic[1] * oldHeader[].head)
  preparedInputs.data = preparedInputs.data + (vk[].ic[2] * oldHeader[].tail)
  preparedInputs.data = preparedInputs.data + (vk[].ic[3] * newHeader[].head)
  preparedInputs.data = preparedInputs.data + (vk[].ic[4] * newHeader[].tail)

  let aBPairing = pairing(prf[].a, prf[].b)
  let alphaBetaPairingP = pairing(vk[].alpha, vk[].beta)
  let preparedInputsGammaPairing = pairing(preparedInputs.data, vk[].gamma)
  let proofCVkDeltaPairing = pairing(prf[].c, vk[].delta)
  let sum = alphaBetaPairingP * preparedInputsGammaPairing * proofCVkDeltaPairing;

  aBPairing == sum


proc makePairsAndVerify*(vk:VerificationKey, prf:Proof, preparedInputs:Input): bool {.wasmPragma.} =

#   # echo $$vk
#   # echo "now alpha"
#   # echo $$vk.alpha
  let aBPairing = pairing(prf.a, prf.b)
  let alphaBetaPairingP = pairing(vk.alpha, vk.beta)
  let preparedInputsGammaPairing = pairing(preparedInputs.data, vk.gamma)
  let proofCVkDeltaPairing = pairing(prf.c, vk.delta)
  let sum = alphaBetaPairingP * preparedInputsGammaPairing * proofCVkDeltaPairing;
  #echo testVerify(1,1,2)
  aBPairing == sum
