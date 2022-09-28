// =================
//  / BASIC TYPES \
// =================

export type Uint64 = TNat;
export type Bit = TNat; // 0 | 1

export type Bytes98 = TBytes;
export type Bytes46 = TBytes;
export type Bytes32 = TBytes;
export type Bytes8 = TBytes;
export type Bytes4 = TBytes;
export type Bytes = TBytes;

export type Slot = Uint64;
export type Epoch = Uint64;

export type ValidatorIndex = Uint64;

export type Root = Bytes32;
export type Domain = Bytes32;
export type DomainType = Bytes4;
export type Version = Bytes4;

export type BLSPubkey = Bytes98;
export type BLSSignature = Bytes46;

export type Bitvector = TList<Bit>;
