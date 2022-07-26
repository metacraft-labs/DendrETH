const
  SIZE_PUBKEY = 32
  NULL = 0
  SUCCESS = 0
  ERROR_CUSTOM_ZERO: uint64 = 1 shl 32
  ERROR_INVALID_ARGUMENT: uint64 = 2 shl 32
  ERROR_INVALID_INSTRUCTION_DATA: uint64 = 3 shl 32
  ERROR_INVALID_ACCOUNT_DATA: uint64 = 4 shl 32
  ERROR_ACCOUNT_DATA_TOO_SMALL: uint64 = 5 shl 32
  ERROR_INSUFFICIENT_FUNDS: uint64 = 6 shl 32
  ERROR_INCORRECT_PROGRAM_ID: uint64 = 7 shl 32
  ERROR_MISSING_REQUIRED_SIGNATURES: uint64 = 8 shl 32
  ERROR_ACCOUNT_ALREADY_INITIALIZED: uint64 = 9 shl 32
  ERROR_UNINITIALIZED_ACCOUNT: uint64 = 10 shl 32
  ERROR_NOT_ENOUGH_ACCOUNT_KEYS: uint64 = 11 shl 32
  ERROR_ACCOUNT_BORROW_FAILED: uint64 = 12 shl 32
  MAX_SEED_LENGTH_EXCEEDED: uint64 = 13 shl 32
  INVALID_SEEDS: uint64 = 14 shl 32

type
  ConstUint8Ptr* {.importc: "const uint8_t*".} = object
  ConstSolPubkeyPtr* {.importc: "const SolPubkey*".} = object

  SolPubKey* {.importc, header: "solana_sdk.h".} = object
    x: array[SIZE_PUBKEY, uint8]

  SolAccountInfo*  {.importc, header: "solana_sdk.h".} = object
    key: ptr SolPubkey
    lamports: ptr uint64
    data_len: uint64
    data: ptr uint8
    owner: ptr SolPubkey
    rent_epoch: uint64
    is_signer: bool
    is_writable: bool
    executable: bool

  SolParameters* {.importc, header: "solana_sdk.h".} = object
    ka: ptr SolAccountInfo
    ka_num: uint64
    data: ConstUint8Ptr
    data_len: uint64
    program_id: ConstSolPubkeyPtr

template SOL_ARRAY_SIZE[T](a: openarray[T]): uint64 =
  (sizeof(a) / sizeof(a[0])).uint64

func sol_log_imp (message: cstring, len: uint64) {.importc: "sol_log_", header: "solana_sdk.h".};
func sol_strlen (s: cstring): uint64 {.importc: "sol_strlen", header: "solana_sdk.h".};
func sol_deserialize (input: ConstUint8Ptr, params: ptr SolParameters, ka_num: uint64): bool {.importc: "sol_deserialize", header: "solana_sdk.h".};
func SolPubkey_same (one: ptr SolPubkey | ConstSolPubkeyPtr, two: ptr SolPubkey | ConstSolPubkeyPtr): bool {.importc: "SolPubkey_same", header: "solana_sdk.h".};

template sol_log (message: cstring) =
  sol_log_imp(message, sol_strlen(message))

func helloworld*(params: ptr SolParameters): uint64 =
  if (params.ka_num < 1):
    sol_log("Greeted account not included in the instruction");
    return ERROR_NOT_ENOUGH_ACCOUNT_KEYS;

  var greeted_account: ptr SolAccountInfo = params.ka;

  if (not SolPubkey_same(greeted_account.owner, params.program_id)):
    sol_log("Greeted account does not have the correct program id");
    return ERROR_INCORRECT_PROGRAM_ID;

  if (greeted_account.data_len < sizeof(uint32).uint64):
    sol_log("Greeted account data length too small to hold uint32_t value");
    return ERROR_INVALID_ACCOUNT_DATA;


  var num_greets = cast[ptr uint32](greeted_account.data);
  num_greets[] += 1.uint32;

  sol_log("Hello!");

  return SUCCESS;

func entrypoint*(input: ConstUint8Ptr): uint64 {.exportc: "entrypoint", codegenDecl: "extern $# $#$#".} =
  sol_log("Helloworld C program entrypoint");
  var accounts: array[1,SolAccountInfo];
  let params: SolParameters = SolParameters(ka: addr accounts[0]);

  if (not sol_deserialize(input, unsafeAddr params, SOL_ARRAY_SIZE(accounts))):
    return ERROR_INVALID_ARGUMENT;

  return helloworld(unsafeAddr params);

