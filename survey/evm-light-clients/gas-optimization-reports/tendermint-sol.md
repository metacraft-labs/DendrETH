# TENDERMINT-SOL
## Gas Optimizations Report


### Use a more recent version of solidity
- Use a solidity version of at least 0.8.3 to get better struct packing and cheaper multiple storage reads \
- Use a solidity version of at least 0.8.4 to get custom errors, which are cheaper at deployment than revert()/require() strings \
- Use a solidity version of at least 0.8.10 to have external calls skip contract existence checks if the external call has a return value

### Functions that are access-restricted from most users may be marked as `payable`
Marking a function as `payable` reduces gas cost since the compiler does not have to check whether a payment was provided or not. This change will save around 21 gas per function call.

_There are **25** instances of this issue:_

```solidity
File: contracts/ibc/IBCHandler.sol

34:   function registerClient(string calldata clientType, IClient client) external {

153:  function bindPort(string memory portId, address moduleAddress) public {

158:  function setExpectedTimePerBlock(uint64 expectedTimePerBlock_) external {
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/ibc/IBCHandler.sol

```solidity
File: contracts/ibc/IBCHost.sol

45:   function setIBCModule(address ibcModule_) external {

58:   function setClientImpl(string calldata clientType, address clientImpl) external {

64:   function getClientImpl(string calldata clientType) external view returns (address, bool) {

70:   function setClientType(string calldata clientId, string calldata clientType) external {

83:   function setClientState(string calldata clientId, bytes calldata clientStateBytes) external {

95:   function setConsensusState(string calldata clientId, uint64 height, bytes calldata consensusStateBytes) external {

107:  function setProcessedTime(string calldata clientId, uint64 height, uint256 processedTime) external {

117:  function setProcessedHeight(string calldata clientId, uint64 height, uint256 processedHeight) external {

129:  function setConnection(string memory connectionId, ConnectionEnd.Data memory connection) public {

148:  function setChannel(string memory portId, string memory channelId, Channel.Data memory channel) public {

160:  function setNextSequenceSend(string calldata portId, string calldata channelId, uint64 sequence) external {

169:  function setNextSequenceRecv(string calldata portId, string calldata channelId, uint64 sequence) external {

178:  function setNextSequenceAck(string calldata portId, string calldata channelId, uint64 sequence) external {

187:  function setPacketCommitment(string memory portId, string memory channelId, uint64 sequence, Packet.Data memory packet) public {

192:  function deletePacketCommitment(string calldata portId, string calldata channelId, uint64 sequence) external {

207:  function setPacketAcknowledgementCommitment(string calldata portId, string calldata channelId, uint64 sequence, bytes calldata acknowledgement) external {

221:  function setPacketReceipt(string calldata portId, string calldata channelId, uint64 sequence) external {

234:  function setExpectedTimePerBlock(uint64 expectedTimePerBlock_) external {

241:  function claimCapability(bytes calldata name, address addr) external {

249:  function authenticateCapability(bytes calldata name, address addr) external view returns (bool) {

268:  function generateClientIdentifier(string calldata clientType) external returns (string memory) {

276:  function generateConnectionIdentifier() external returns (string memory) {

284:  function generateChannelIdentifier() external returns (string memory) {
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/ibc/IBCHost.sol

### Usage of `uint`s/`int`s smaller than 32 bytes (256 bits) incurs overhead
'When using elements that are smaller than 32 bytes, your contract’s gas usage may be higher. This is because the EVM operates on 32 bytes at a time. Therefore, if the element is smaller than that, the EVM must use more operations in order to reduce the size of the element from 32 bytes to the desired size.' \ https://docs.soliditylang.org/en/v0.8.15/internals/layout_in_storage.html \ Use a larger size then downcast where needed

_There are **293** instances of this issue:_

### Don't compare boolean expressions to boolean literals
Use `if(x)`/`if(!x)` instead of `if(x == true)`/`if(x == false)`.

_There are **32** instances of this issue:_

```solidity
File: contracts/ics23/ics23.sol

68:   if (ExistenceProof.isNil(proof.exist) == false){

69:   if (BytesLib.equal(proof.exist.key, key) == true) {

72:   } else if(BatchProof.isNil(proof.batch) == false) {

74:   if (ExistenceProof.isNil(proof.batch.entries[i].exist) == false &&

84:   if (NonExistenceProof.isNil(proof.nonexist) == false) {

88:   } else if (BatchProof.isNil(proof.batch) == false) {

90:   if (NonExistenceProof.isNil(proof.batch.entries[i].nonexist) == false &&
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/ics23/ics23.sol

```solidity
File: contracts/ics23/ics23Compress.sol

12:   if (CompressedBatchProof._empty(proof.compressed) == true){

36:   if (CompressedExistenceProof._empty(entry.exist) == false) {
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/ics23/ics23Compress.sol

```solidity
File: contracts/ics23/ics23Ops.sol

74:   if (hasprefix == false) return CheckAgainstSpecError.HasPrefix;
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/ics23/ics23Ops.sol

```solidity
File: contracts/ics23/ics23Proof.sol

28:   if (keyMatch == false) return VerifyExistenceError.KeyNotMatching;

31:   if (valueMatch == false) return VerifyExistenceError.ValueNotMatching;

38:   if (rootMatch == false) return VerifyExistenceError.RootNotMatching;

111:  if (ExistenceProof._empty(proof.left) == false) {

117:  if (ExistenceProof._empty(proof.right) == false) {

137:  if(isLeftMost(spec.inner_spec, proof.right.path) == false) return VerifyNonExistenceError.RightProofLeftMost;

140:  if (isRightMost(spec.inner_spec, proof.left.path) == false) return VerifyNonExistenceError.LeftProofRightMost;

144:  if (isLeftNeigh == false) return VerifyNonExistenceError.IsLeftNeighbor;

151:  if (ExistenceProof._empty(proof.left) == false) {

154:  if (ExistenceProof._empty(proof.right) == false) {

163:  if (ExistenceProof._empty(proof.exist) == false) {

166:  if (NonExistenceProof._empty(proof.nonexist) == false) {

169:  if (BatchProof._empty(proof.batch) == false) {

174:  if (ExistenceProof._empty(proof.batch.entries[0].exist) == false) {

177:  if (NonExistenceProof._empty(proof.batch.entries[0].nonexist) == false) {

181:  if (CompressedBatchProof._empty(proof.compressed) == false) {

194:  if (hasPadding(path[i], minPrefix, maxPrefix, suffix) == false){

206:  if (hasPadding(path[i], minPrefix, maxPrefix, suffix) == false){

236:  if (isLeftStep(spec, left[leftIdx], right[rightIdx]) == false) {

240:  if (isRightMost(spec, sliceInnerOps(left, 0, leftIdx)) == false){

243:  if (isLeftMost(spec, sliceInnerOps(right, 0, rightIdx)) == false) {

259:  if (hasPadding(op, minp, maxp, suffix) == true) return (branch, OrderFromPaddingError.None);
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/ics23/ics23Proof.sol

### Splitting `require()` statements that use `&&` saves gas
Instead of using `&&` on single `require` check using two `require` checks can save gas

_There are **1** instances of this issue:_

```solidity
File: contracts/ibc/IBCChannel.sol

273:  require(nextSequenceRecv > 0 && nextSequenceRecv == msg_.packet.sequence, "packet sequence next receive sequence");
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/ibc/IBCChannel.sol

### Replace `x <= y` with `x < y + 1`, and `x >= y` with `x > y - 1`
In the EVM, there is no opcode for `>=` or `<=`. When using greater than or equal, two operations are performed: `>` and `=`. Using strict comparison operators hence saves gas

_There are **14** instances of this issue:_

```solidity
File: contracts/ics23/ics23Compress.sol

63:   require(proof.path[i] >= 0);
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/ics23/ics23Compress.sol

```solidity
File: contracts/ics23/ics23Proof.sol

127:  if (rightKey.length > 0 && Ops.compare(key, rightKey) >= 0) {

131:  if (leftKey.length > 0 && Ops.compare(key, leftKey) <= 0) {

227:  while (leftIdx >= 0 && rightIdx >= 0) {

288:  if (branch >= order.length) return (0, GetPositionError.BranchLength);
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/ics23/ics23Proof.sol

```solidity
File: contracts/proto/ProtoBufRuntime.sol

116:  for (; len >= WORD_LENGTH; len -= WORD_LENGTH) {

487:  assert(p + sz <= length);

570:  assert(p + sz + len <= length);

863:  if (i >= 0) {

2872: if (x >= 0) {

2876: if (remainder >= 128) {
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/proto/ProtoBufRuntime.sol

```solidity
File: contracts/proto/TendermintHelper.sol

170:  require(sum <= maxTotalVotingPower, "total voting power should be guarded to not exceed");
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/proto/TendermintHelper.sol

```solidity
File: contracts/utils/Bytes.sol

17:   require(_bytes.length >= _start + 8, "Bytes: toUint64 out of bounds");

24:   require(_bytes.length >= _start + 32, "Bytes: toUint256 out of bounds");
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/utils/Bytes.sol

### Use `immutable` & `constant` for state variables that do not change their value

_There are **5** instances of this issue:_

```solidity
File: contracts/ibc/IBCHandler.sol

16:   address owner;

17:   IBCHost host;
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/ibc/IBCHandler.sol

```solidity
File: contracts/ibc/IBCHost.sol

36:   address owner;

37:   address ibcModule;
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/ibc/IBCHost.sol

```solidity
File: contracts/TendermintLightClient.sol

43:   ProtoTypes private _pts;
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/TendermintLightClient.sol


### The usage of `++i` will cost less gas than `i++`. The same change can be applied to `i--` as well.
This change would save up to 6 gas per instance/loop.

_There are **74** instances of this issue:_

```solidity
File: contracts/ibc/IBCChannel.sol

236:  nextSequenceSend++;

327:  nextSequenceAck++;
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/ibc/IBCChannel.sol

```solidity
File: contracts/ibc/IBCHost.sol

135:  for (uint8 i = 0; i < connection.versions.length; i++) {

243:  for (uint32 i = 0; i < capabilities[name].length; i++) {

251:  for (uint32 i = 0; i < capabilities[name].length; i++) {

271:  nextClientSequence++;

279:  nextConnectionSequence++;

287:  nextChannelSequence++;

299:  len++;
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/ibc/IBCHost.sol

```solidity
File: contracts/ics23/ics23.sol

73:   for (uint i = 0; i < proof.batch.entries.length; i++) {

89:   for (uint i = 0; i < proof.batch.entries.length; i++) {
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/ics23/ics23.sol

```solidity
File: contracts/ics23/ics23Compress.sol

28:   for(uint i = 0; i < proof.entries.length; i++) {

62:   for (uint i = 0; i < proof.path.length; i++) {
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/ics23/ics23Compress.sol

```solidity
File: contracts/ics23/ics23Ops.sol

155:  for (uint i = 0; i < minLen; i++) {
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/ics23/ics23Ops.sol

```solidity
File: contracts/ics23/ics23Proof.sol

57:   for (uint i = 0; i < proof.path.length; i++) {

89:   for(uint i = 0; i < proof.path.length; i++) {

193:  for (uint i = 0; i < path.length; i++) {

205:  for (uint i = 0; i < path.length; i++) {

256:  for(uint branch = 0; branch < maxBranch; branch++) {

289:  for (uint i = 0; i < order.length; i++) {

304:  for (uint i = start; i < end; i++) {
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/ics23/ics23Proof.sol

```solidity
File: contracts/proto/Channel.sol

479:  for(i = 0; i < r.connection_hops.length; i++) {

544:  for(i = 0; i < r.connection_hops.length; i++) {

603:  for (uint256 i = 0; i < self.connection_hops.length; i++) {

1369: for(i = 0; i < r.connection_hops.length; i++) {

1452: for(i = 0; i < r.connection_hops.length; i++) {

1523: for (uint256 i = 0; i < self.connection_hops.length; i++) {
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/proto/Channel.sol

```solidity
File: contracts/proto/Connection.sol

424:  for(i = 0; i < r.versions.length; i++) {

506:  for(i = 0; i < r.versions.length; i++) {

549:  for(uint256 i2 = 0; i2 < input.versions.length; i2++) {

571:  for (uint256 i = 0; i < self.versions.length; i++) {

1440: for(i = 0; i < r.features.length; i++) {

1494: for(i = 0; i < r.features.length; i++) {

1541: for (uint256 i = 0; i < self.features.length; i++) {
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/proto/Connection.sol

```solidity
File: contracts/proto/proofs.sol

354:  for(i = 0; i < r.path.length; i++) {

410:  for(i = 0; i < r.path.length; i++) {

448:  for(uint256 i4 = 0; i4 < input.path.length; i4++) {

467:  for (uint256 i = 0; i < self.path.length; i++) {

2873: for(i = 0; i < r.child_order.length; i++) {

2972: for(i = 0; i < r.child_order.length; i++) {

3044: for (uint256 i = 0; i < self.child_order.length; i++) {

3276: for(i = 0; i < r.entries.length; i++) {

3329: for(i = 0; i < r.entries.length; i++) {

3356: for(uint256 i1 = 0; i1 < input.entries.length; i1++) {

3375: for (uint256 i = 0; i < self.entries.length; i++) {

3991: for(i = 0; i < r.entries.length; i++) {

4002: for(i = 0; i < r.lookup_inners.length; i++) {

4055: for(i = 0; i < r.entries.length; i++) {

4058: for(i = 0; i < r.lookup_inners.length; i++) {

4089: for(uint256 i1 = 0; i1 < input.entries.length; i1++) {

4094: for(uint256 i2 = 0; i2 < input.lookup_inners.length; i2++) {

4113: for (uint256 i = 0; i < self.entries.length; i++) {

4131: for (uint256 i = 0; i < self.lookup_inners.length; i++) {

4822: for(i = 0; i < r.path.length; i++) {

4878: for(i = 0; i < r.path.length; i++) {

4931: for (uint256 i = 0; i < self.path.length; i++) {
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/proto/proofs.sol

```solidity
File: contracts/proto/ProtoBufRuntime.sol

50:   for (uint256 i = 0; i < ceil(length, WORD_LENGTH); i++) {

94:   for (uint256 i = 0; i < ceil(length, WORD_LENGTH); i++) {

980:  for (uint256 i = len - 2; i < WORD_LENGTH; i++) {

2916: for (uint256 i = 0; i < sz; i++) {

2919: actualSize--;
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/proto/ProtoBufRuntime.sol

```solidity
File: contracts/proto/TendermintHelper.sol

153:  for (uint256 idx; idx < vals.validators.length; idx++) {

168:  for (uint256 i = 0; i < vals.validators.length; i++) {
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/proto/TendermintHelper.sol

```solidity
File: contracts/proto/TendermintLight.sol

4182: for(i = 0; i < r.validators.length; i++) {

4253: for(i = 0; i < r.validators.length; i++) {

4286: for(uint256 i1 = 0; i1 < input.validators.length; i1++) {

4307: for (uint256 i = 0; i < self.validators.length; i++) {

6368: for(i = 0; i < r.signatures.length; i++) {

6424: for(i = 0; i < r.signatures.length; i++) {

6462: for(uint256 i4 = 0; i4 < input.signatures.length; i4++) {

6481: for (uint256 i = 0; i < self.signatures.length; i++) {
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/proto/TendermintLight.sol

```solidity
File: contracts/utils/crypto/MerkleTree.sol

44:   for (uint i = input - 1; i > 1; i--) {
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/utils/crypto/MerkleTree.sol

```solidity
File: contracts/utils/Tendermint.sol

184:  for (uint256 idx = 0; idx < commit.signatures.length; idx++) {

244:  for (uint256 i = 0; i < commit.signatures.length; i++) {
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/utils/Tendermint.sol

### Using `!= 0` on `uints` costs less gas than `> 0`.
This change saves 3 gas per instance/loop

_There are **5** instances of this issue:_

```solidity
File: contracts/ibc/IBCChannel.sol

233:  require(nextSequenceSend > 0, "sequenceSend not found");

273:  require(nextSequenceRecv > 0 && nextSequenceRecv == msg_.packet.sequence, "packet sequence next receive sequence");
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/ibc/IBCChannel.sol

```solidity
File: contracts/mocks/MerkleTreeMock.sol

17:   if (total > 0) {
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/mocks/MerkleTreeMock.sol

```solidity
File: contracts/proto/ProtoBufRuntime.sol

2850: x & (base << (realSize * BYTE_SIZE - BYTE_SIZE)) == 0 && realSize > 0

2886: realSize > 0
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/proto/ProtoBufRuntime.sol

### It costs more gas to initialize non-`constant`/non-`immutable` variables to zero than to let the default of zero be applied
Not overwriting the default for stack variables saves 8 gas. Storage and memory variables have larger savings

_There are **48** instances of this issue:_

```solidity
File: contracts/ibc/IBCConnection.sol

187:  uint64 blockDelay = 0;
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/ibc/IBCConnection.sol

```solidity
File: contracts/ibc/IBCHost.sol

135:  for (uint8 i = 0; i < connection.versions.length; i++) {

243:  for (uint32 i = 0; i < capabilities[name].length; i++) {

251:  for (uint32 i = 0; i < capabilities[name].length; i++) {
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/ibc/IBCHost.sol

```solidity
File: contracts/ics23/ics23.sol

73:   for (uint i = 0; i < proof.batch.entries.length; i++) {

89:   for (uint i = 0; i < proof.batch.entries.length; i++) {
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/ics23/ics23.sol

```solidity
File: contracts/ics23/ics23Compress.sol

28:   for(uint i = 0; i < proof.entries.length; i++) {

62:   for (uint i = 0; i < proof.path.length; i++) {
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/ics23/ics23Compress.sol

```solidity
File: contracts/ics23/ics23Ops.sol

155:  for (uint i = 0; i < minLen; i++) {
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/ics23/ics23Ops.sol

```solidity
File: contracts/ics23/ics23Proof.sol

57:   for (uint i = 0; i < proof.path.length; i++) {

89:   for(uint i = 0; i < proof.path.length; i++) {

193:  for (uint i = 0; i < path.length; i++) {

205:  for (uint i = 0; i < path.length; i++) {

256:  for(uint branch = 0; branch < maxBranch; branch++) {

289:  for (uint i = 0; i < order.length; i++) {
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/ics23/ics23Proof.sol

```solidity
File: contracts/proto/Channel.sol

603:  for (uint256 i = 0; i < self.connection_hops.length; i++) {

1523: for (uint256 i = 0; i < self.connection_hops.length; i++) {
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/proto/Channel.sol

```solidity
File: contracts/proto/Connection.sol

549:  for(uint256 i2 = 0; i2 < input.versions.length; i2++) {

571:  for (uint256 i = 0; i < self.versions.length; i++) {

1541: for (uint256 i = 0; i < self.features.length; i++) {
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/proto/Connection.sol

```solidity
File: contracts/proto/proofs.sol

448:  for(uint256 i4 = 0; i4 < input.path.length; i4++) {

467:  for (uint256 i = 0; i < self.path.length; i++) {

3044: for (uint256 i = 0; i < self.child_order.length; i++) {

3356: for(uint256 i1 = 0; i1 < input.entries.length; i1++) {

3375: for (uint256 i = 0; i < self.entries.length; i++) {

4089: for(uint256 i1 = 0; i1 < input.entries.length; i1++) {

4094: for(uint256 i2 = 0; i2 < input.lookup_inners.length; i2++) {

4113: for (uint256 i = 0; i < self.entries.length; i++) {

4131: for (uint256 i = 0; i < self.lookup_inners.length; i++) {

4931: for (uint256 i = 0; i < self.path.length; i++) {
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/proto/proofs.sol

```solidity
File: contracts/proto/ProtoBufRuntime.sol

50:   for (uint256 i = 0; i < ceil(length, WORD_LENGTH); i++) {

94:   for (uint256 i = 0; i < ceil(length, WORD_LENGTH); i++) {

392:  uint256 x = 0;

393:  uint256 sz = 0;

485:  uint256 x = 0;

617:  uint256 sz = 0;

671:  uint256 count = 0;

2916: for (uint256 i = 0; i < sz; i++) {
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/proto/ProtoBufRuntime.sol

```solidity
File: contracts/proto/TendermintHelper.sol

164:  uint256 sum = 0;

168:  for (uint256 i = 0; i < vals.validators.length; i++) {
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/proto/TendermintHelper.sol

```solidity
File: contracts/proto/TendermintLight.sol

4286: for(uint256 i1 = 0; i1 < input.validators.length; i1++) {

4307: for (uint256 i = 0; i < self.validators.length; i++) {

6462: for(uint256 i4 = 0; i4 < input.signatures.length; i4++) {

6481: for (uint256 i = 0; i < self.signatures.length; i++) {
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/proto/TendermintLight.sol

```solidity
File: contracts/utils/Tendermint.sol

177:  int64 talliedVotingPower = 0;

184:  for (uint256 idx = 0; idx < commit.signatures.length; idx++) {

241:  int64 talliedVotingPower = 0;

244:  for (uint256 i = 0; i < commit.signatures.length; i++) {
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/utils/Tendermint.sol

### `++i`/`i++` should be `unchecked{++I}`/`unchecked{I++}` in `for`-loops
When an increment or any arithmetic operation is not possible to overflow it should be placed in `unchecked{}` block. \This is because of the default compiler overflow and underflow safety checks since Solidity version 0.8.0. \In for-loops it saves around 30-40 gas **per loop**

_There are **66** instances of this issue:_

```solidity
File: contracts/ibc/IBCHost.sol

135:  for (uint8 i = 0; i < connection.versions.length; i++) {

243:  for (uint32 i = 0; i < capabilities[name].length; i++) {

251:  for (uint32 i = 0; i < capabilities[name].length; i++) {
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/ibc/IBCHost.sol

```solidity
File: contracts/ics23/ics23.sol

73:   for (uint i = 0; i < proof.batch.entries.length; i++) {

89:   for (uint i = 0; i < proof.batch.entries.length; i++) {
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/ics23/ics23.sol

```solidity
File: contracts/ics23/ics23Compress.sol

28:   for(uint i = 0; i < proof.entries.length; i++) {

62:   for (uint i = 0; i < proof.path.length; i++) {
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/ics23/ics23Compress.sol

```solidity
File: contracts/ics23/ics23Ops.sol

155:  for (uint i = 0; i < minLen; i++) {
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/ics23/ics23Ops.sol

```solidity
File: contracts/ics23/ics23Proof.sol

57:   for (uint i = 0; i < proof.path.length; i++) {

89:   for(uint i = 0; i < proof.path.length; i++) {

193:  for (uint i = 0; i < path.length; i++) {

205:  for (uint i = 0; i < path.length; i++) {

256:  for(uint branch = 0; branch < maxBranch; branch++) {

289:  for (uint i = 0; i < order.length; i++) {

304:  for (uint i = start; i < end; i++) {
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/ics23/ics23Proof.sol

```solidity
File: contracts/proto/Channel.sol

479:  for(i = 0; i < r.connection_hops.length; i++) {

544:  for(i = 0; i < r.connection_hops.length; i++) {

603:  for (uint256 i = 0; i < self.connection_hops.length; i++) {

1369: for(i = 0; i < r.connection_hops.length; i++) {

1452: for(i = 0; i < r.connection_hops.length; i++) {

1523: for (uint256 i = 0; i < self.connection_hops.length; i++) {
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/proto/Channel.sol

```solidity
File: contracts/proto/Connection.sol

424:  for(i = 0; i < r.versions.length; i++) {

506:  for(i = 0; i < r.versions.length; i++) {

549:  for(uint256 i2 = 0; i2 < input.versions.length; i2++) {

571:  for (uint256 i = 0; i < self.versions.length; i++) {

1440: for(i = 0; i < r.features.length; i++) {

1494: for(i = 0; i < r.features.length; i++) {

1541: for (uint256 i = 0; i < self.features.length; i++) {
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/proto/Connection.sol

```solidity
File: contracts/proto/proofs.sol

354:  for(i = 0; i < r.path.length; i++) {

410:  for(i = 0; i < r.path.length; i++) {

448:  for(uint256 i4 = 0; i4 < input.path.length; i4++) {

467:  for (uint256 i = 0; i < self.path.length; i++) {

2873: for(i = 0; i < r.child_order.length; i++) {

2972: for(i = 0; i < r.child_order.length; i++) {

3044: for (uint256 i = 0; i < self.child_order.length; i++) {

3276: for(i = 0; i < r.entries.length; i++) {

3329: for(i = 0; i < r.entries.length; i++) {

3356: for(uint256 i1 = 0; i1 < input.entries.length; i1++) {

3375: for (uint256 i = 0; i < self.entries.length; i++) {

3991: for(i = 0; i < r.entries.length; i++) {

4002: for(i = 0; i < r.lookup_inners.length; i++) {

4055: for(i = 0; i < r.entries.length; i++) {

4058: for(i = 0; i < r.lookup_inners.length; i++) {

4089: for(uint256 i1 = 0; i1 < input.entries.length; i1++) {

4094: for(uint256 i2 = 0; i2 < input.lookup_inners.length; i2++) {

4113: for (uint256 i = 0; i < self.entries.length; i++) {

4131: for (uint256 i = 0; i < self.lookup_inners.length; i++) {

4822: for(i = 0; i < r.path.length; i++) {

4878: for(i = 0; i < r.path.length; i++) {

4931: for (uint256 i = 0; i < self.path.length; i++) {
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/proto/proofs.sol

```solidity
File: contracts/proto/ProtoBufRuntime.sol

50:   for (uint256 i = 0; i < ceil(length, WORD_LENGTH); i++) {

94:   for (uint256 i = 0; i < ceil(length, WORD_LENGTH); i++) {

980:  for (uint256 i = len - 2; i < WORD_LENGTH; i++) {

2916: for (uint256 i = 0; i < sz; i++) {
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/proto/ProtoBufRuntime.sol

```solidity
File: contracts/proto/TendermintHelper.sol

153:  for (uint256 idx; idx < vals.validators.length; idx++) {

168:  for (uint256 i = 0; i < vals.validators.length; i++) {
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/proto/TendermintHelper.sol

```solidity
File: contracts/proto/TendermintLight.sol

4182: for(i = 0; i < r.validators.length; i++) {

4253: for(i = 0; i < r.validators.length; i++) {

4286: for(uint256 i1 = 0; i1 < input.validators.length; i1++) {

4307: for (uint256 i = 0; i < self.validators.length; i++) {

6368: for(i = 0; i < r.signatures.length; i++) {

6424: for(i = 0; i < r.signatures.length; i++) {

6462: for(uint256 i4 = 0; i4 < input.signatures.length; i4++) {

6481: for (uint256 i = 0; i < self.signatures.length; i++) {
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/proto/TendermintLight.sol

```solidity
File: contracts/utils/Tendermint.sol

184:  for (uint256 idx = 0; idx < commit.signatures.length; idx++) {

244:  for (uint256 i = 0; i < commit.signatures.length; i++) {
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/utils/Tendermint.sol

### `require()/revert()` strings longer than 32 bytes cost extra gas
Each extra memory word of bytes past the original 32 incurs an MSTORE which costs 3 gas

_There are **45** instances of this issue:_

```solidity
File: contracts/ibc/IBCChannel.sol

23:   require(connection.versions.length == 1, "single version must be negotiated on connection before opening channel");

49:   require(connection.versions.length == 1, "single version must be negotiated on connection before opening channel");

50:   require(msg_.channel.state == Channel.State.STATE_TRYOPEN, "channel state must be STATE_TRYOPEN");

221:  require(hashString(packet.destination_port) == hashString(channel.counterparty.port_id), "packet destination port doesn't match the counterparty's port");

222:  require(hashString(packet.destination_channel) == hashString(channel.counterparty.channel_id), "packet destination channel doesn't match the counterparty's channel");

227:  require(packet.timeout_height.revision_height == 0 || latestHeight < packet.timeout_height.revision_height, "receiving chain block height >= packet timeout height");

230:  require(packet.timeout_timestamp == 0 || latestTimestamp < packet.timeout_timestamp, "receiving chain block timestamp >= packet timeout timestamp");

234:  require(packet.sequence == nextSequenceSend, "packet sequence next send sequence");

255:  require(hashString(msg_.packet.source_port) == hashString(channel.counterparty.port_id), "packet source port doesn't match the counterparty's port");

256:  require(hashString(msg_.packet.source_channel) == hashString(channel.counterparty.channel_id), "packet source channel doesn't match the counterparty's channel");

262:  require(msg_.packet.timeout_height.revision_height == 0 || block.number < msg_.packet.timeout_height.revision_height, "block height >= packet timeout height");

263:  require(msg_.packet.timeout_timestamp == 0 || block.timestamp < msg_.packet.timeout_timestamp, "block timestamp >= packet timeout timestamp");

266:  require(IBCConnection.verifyPacketCommitment(host, connection, msg_.proofHeight, msg_.proof, msg_.packet.source_port, msg_.packet.source_channel, msg_.packet.sequence, commitment), "failed to verify packet commitment");

269:  require(!host.hasPacketReceipt(msg_.packet.destination_port, msg_.packet.destination_channel, msg_.packet.sequence), "packet sequence already has been received");

273:  require(nextSequenceRecv > 0 && nextSequenceRecv == msg_.packet.sequence, "packet sequence next receive sequence");

292:  require(!found, "acknowledgement for packet already exists");

309:  require(hashString(msg_.packet.destination_port) == hashString(channel.counterparty.port_id), "packet destination port doesn't match the counterparty's port");

310:  require(hashString(msg_.packet.destination_channel) == hashString(channel.counterparty.channel_id), "packet destination channel doesn't match the counterparty's channel");

321:  require(IBCConnection.verifyPacketAcknowledgement(host, connection, msg_.proofHeight, msg_.proof, msg_.packet.destination_port, msg_.packet.destination_channel, msg_.packet.sequence, msg_.acknowledgement), "failed to verify packet acknowledgement commitment");

326:  require(msg_.packet.sequence == nextSequenceAck, "packet sequence next ack sequence");
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/ibc/IBCChannel.sol

```solidity
File: contracts/ibc/IBCConnection.sol

40:   require(IBCClient.validateSelfClient(host, msg_.clientStateBytes), "failed to validate self client state");

41:   require(msg_.counterpartyVersions.length > 0, "counterpartyVersions length must be greater than 0");

63:   require(verifyConnectionState(host, connection, msg_.proofHeight, msg_.proofInit, msg_.counterparty.connection_id, expectedConnection), "failed to verify connection state");

81:   revert("connection state is not INIT or TRYOPEN");

83:   revert("connection state is in INIT but the provided version is not supported");

85:   revert("connection state is in TRYOPEN but the provided version is not set in the previous connection versions");

88:   require(IBCClient.validateSelfClient(host, msg_.clientStateBytes), "failed to validate self client state");

104:  require(verifyConnectionState(host, connection, msg_.proofHeight, msg_.proofTry, msg_.counterpartyConnectionID, expectedConnection), "failed to verify connection state");

138:  require(verifyConnectionState(host, connection, msg_.proofHeight, msg_.proofAck, connection.counterparty.connection_id, expectedConnection), "failed to verify connection state");
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/ibc/IBCConnection.sol

```solidity
File: contracts/ibc/IBCHost.sol

73:   require(bytes(clientType).length > 0, "clientType must not be empty string");
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/ibc/IBCHost.sol

```solidity
File: contracts/mocks/MerkleTreeMock.sol

16:   require(vs.validators.length == total, "requested vs provided validator size differ");

18:   require(vs.validators[0].pub_key.ed25519.length > 0, "expected ed25519 public key, got empty array");
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/mocks/MerkleTreeMock.sol

```solidity
File: contracts/proto/TendermintHelper.sol

170:  require(sum <= maxTotalVotingPower, "total voting power should be guarded to not exceed");
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/proto/TendermintHelper.sol

```solidity
File: contracts/TendermintLightClient.sol

137:  require(ok, "LC: consensusState not found at trusted height");

172:  require(
173:      tmHeader.signed_header.header.height > tmHeader.trusted_height,
174:      "LC: header height consensus state height"
175:  );
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/TendermintLightClient.sol

```solidity
File: contracts/utils/Tendermint.sol

57:   require(untrustedHeader.header.height == trustedHeader.header.height + 1, "headers must be adjacent in height");

64:   require(
65:       untrustedHeader.header.validators_hash.toBytes32() == trustedHeader.header.next_validators_hash.toBytes32(),
66:       "expected old header next validators to match those from new header"
67:   );

91:   require(
92:       untrustedHeader.header.height != trustedHeader.header.height + 1,
93:       "LC: headers must be non adjacent in height"
94:   );

98:   require(
99:       trustedVals.hash() == trustedHeader.header.next_validators_hash.toBytes32(),
100:      "LC: headers trusted validators does not hash to latest trusted validators"
101:  );

135:  require(untrustedHeader.commit.height == untrustedHeader.header.height, "header and commit height mismatch");

143:  require(
144:      untrustedHeader.header.height > trustedHeader.header.height,
145:      "expected new header height to be greater than one of old header"
146:  );

147:  require(
148:      untrustedHeader.header.time.gt(trustedHeader.header.time),
149:      "expected new header time to be after old header time"
150:  );

151:  require(
152:      Timestamp
153:          .Data({
154:              Seconds: int64(currentTime.Seconds) + int64(maxClockDrift.Seconds),
155:              nanos: int32(currentTime.nanos) + int32(maxClockDrift.nanos)
156:          })
157:          .gt(untrustedHeader.header.time),
158:      "new header has time from the future"
159:  );

162:  require(
163:      untrustedHeader.header.validators_hash.toBytes32() == validatorsHash,
164:      "expected new header validators to match those that were supplied at height XX"
165:  );

197:  require(!seenVals[valIdx], "double vote of validator on the same commit");
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/utils/Tendermint.sol

### Use custom errors rather than `revert()`/`require()` strings to save gas
Custom errors are available from solidity version 0.8.4. Custom errors save ~50 gas each time they're hitby avoiding having to allocate and store the revert string. Not defining the strings also save deployment gas

_There are **129** instances of this issue:_

```solidity
File: contracts/ibc/IBCChannel.sol

20:   require(msg_.channel.connection_hops.length == 1, "connection_hops length must be 1");

22:   require(found, "connection not found");

23:   require(connection.versions.length == 1, "single version must be negotiated on connection before opening channel");

24:   require(msg_.channel.state == Channel.State.STATE_INIT, "channel state must STATE_INIT");

46:   require(msg_.channel.connection_hops.length == 1, "connection_hops length must be 1");

48:   require(found, "connection not found");

49:   require(connection.versions.length == 1, "single version must be negotiated on connection before opening channel");

50:   require(msg_.channel.state == Channel.State.STATE_TRYOPEN, "channel state must be STATE_TRYOPEN");

67:   require(IBCConnection.verifyChannelState(host, connection, msg_.proofHeight, msg_.proofInit, msg_.channel.counterparty.port_id, msg_.channel.counterparty.channel_id, Channel.encode(expectedChannel)), "failed to verify channel state");

88:   require(found, "channel not found");

89:   require(channel.state == Channel.State.STATE_INIT || channel.state == Channel.State.STATE_TRYOPEN, "invalid channel state");

94:   require(found, "connection not found");

95:   require(connection.state == ConnectionEnd.State.STATE_OPEN, "connection state is not OPEN");

108:  require(IBCConnection.verifyChannelState(host, connection, msg_.proofHeight, msg_.proofTry, channel.counterparty.port_id, msg_.counterpartyChannelId, Channel.encode(expectedChannel)), "failed to verify channel state");

125:  require(found, "channel not found");

126:  require(channel.state == Channel.State.STATE_TRYOPEN, "channel state is not TRYOPEN");

131:  require(found, "connection not found");

132:  require(connection.state == ConnectionEnd.State.STATE_OPEN, "connection state is not OPEN");

145:  require(IBCConnection.verifyChannelState(host, connection, msg_.proofHeight, msg_.proofAck, channel.counterparty.port_id, channel.counterparty.channel_id, Channel.encode(expectedChannel)), "failed to verify channel state");

160:  require(found, "channel not found");

161:  require(channel.state != Channel.State.STATE_CLOSED, "channel state is already CLOSED");

166:  require(found, "connection not found");

167:  require(connection.state == ConnectionEnd.State.STATE_OPEN, "connection state is not OPEN");

183:  require(found, "channel not found");

184:  require(channel.state != Channel.State.STATE_CLOSED, "channel state is already CLOSED");

189:  require(found, "connection not found");

190:  require(connection.state == ConnectionEnd.State.STATE_OPEN, "connection state is not OPEN");

203:  require(IBCConnection.verifyChannelState(host, connection, msg_.proofHeight, msg_.proofInit, channel.counterparty.port_id, channel.counterparty.channel_id, Channel.encode(expectedChannel)), "failed to verify channel state");

219:  require(found, "channel not found");

220:  require(channel.state == Channel.State.STATE_OPEN, "channel state must be OPEN");

221:  require(hashString(packet.destination_port) == hashString(channel.counterparty.port_id), "packet destination port doesn't match the counterparty's port");

222:  require(hashString(packet.destination_channel) == hashString(channel.counterparty.channel_id), "packet destination channel doesn't match the counterparty's channel");

224:  require(found, "connection not found");

227:  require(packet.timeout_height.revision_height == 0 || latestHeight < packet.timeout_height.revision_height, "receiving chain block height >= packet timeout height");

229:  require(found, "consensusState not found");

230:  require(packet.timeout_timestamp == 0 || latestTimestamp < packet.timeout_timestamp, "receiving chain block timestamp >= packet timeout timestamp");

233:  require(nextSequenceSend > 0, "sequenceSend not found");

234:  require(packet.sequence == nextSequenceSend, "packet sequence next send sequence");

249:  require(found, "channel not found");

250:  require(channel.state == Channel.State.STATE_OPEN, "channel state must be OPEN");

255:  require(hashString(msg_.packet.source_port) == hashString(channel.counterparty.port_id), "packet source port doesn't match the counterparty's port");

256:  require(hashString(msg_.packet.source_channel) == hashString(channel.counterparty.channel_id), "packet source channel doesn't match the counterparty's channel");

259:  require(found, "connection not found");

260:  require(connection.state == ConnectionEnd.State.STATE_OPEN, "connection state is not OPEN");

262:  require(msg_.packet.timeout_height.revision_height == 0 || block.number < msg_.packet.timeout_height.revision_height, "block height >= packet timeout height");

263:  require(msg_.packet.timeout_timestamp == 0 || block.timestamp < msg_.packet.timeout_timestamp, "block timestamp >= packet timeout timestamp");

266:  require(IBCConnection.verifyPacketCommitment(host, connection, msg_.proofHeight, msg_.proof, msg_.packet.source_port, msg_.packet.source_channel, msg_.packet.sequence, commitment), "failed to verify packet commitment");

269:  require(!host.hasPacketReceipt(msg_.packet.destination_port, msg_.packet.destination_channel, msg_.packet.sequence), "packet sequence already has been received");

273:  require(nextSequenceRecv > 0 && nextSequenceRecv == msg_.packet.sequence, "packet sequence next receive sequence");

276:  revert("unknown ordering type");

288:  require(found, "channel not found");

289:  require(channel.state == Channel.State.STATE_OPEN, "channel state must be OPEN");

292:  require(!found, "acknowledgement for packet already exists");

294:  require(acknowledgement.length > 0, "acknowledgement cannot be empty");

306:  require(found, "channel not found");

307:  require(channel.state == Channel.State.STATE_OPEN, "channel state must be OPEN");

309:  require(hashString(msg_.packet.destination_port) == hashString(channel.counterparty.port_id), "packet destination port doesn't match the counterparty's port");

310:  require(hashString(msg_.packet.destination_channel) == hashString(channel.counterparty.channel_id), "packet destination channel doesn't match the counterparty's channel");

313:  require(found, "connection not found");

314:  require(connection.state == ConnectionEnd.State.STATE_OPEN, "connection state is not OPEN");

317:  require(found, "packet commitment not found");

319:  require(commitment == host.makePacketCommitment(msg_.packet), "commitment bytes are not equal");

321:  require(IBCConnection.verifyPacketAcknowledgement(host, connection, msg_.proofHeight, msg_.proof, msg_.packet.destination_port, msg_.packet.destination_channel, msg_.packet.sequence, msg_.acknowledgement), "failed to verify packet acknowledgement commitment");

325:  require(nextSequenceAck == 0, "sequence ack not found");

326:  require(msg_.packet.sequence == nextSequenceAck, "packet sequence next ack sequence");

335:  require(channel.connection_hops.length == 1, "connection_hops length must be 1");

337:  require(found, "connection not found");
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/ibc/IBCChannel.sol

```solidity
File: contracts/ibc/IBCClient.sol

18:   require(found, "unregistered client type");

39:   require(found, "clientState not found");

62:   require(found, "clientImpl not found");
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/ibc/IBCClient.sol

```solidity
File: contracts/ibc/IBCConnection.sol

40:   require(IBCClient.validateSelfClient(host, msg_.clientStateBytes), "failed to validate self client state");

41:   require(msg_.counterpartyVersions.length > 0, "counterpartyVersions length must be greater than 0");

63:   require(verifyConnectionState(host, connection, msg_.proofHeight, msg_.proofInit, msg_.counterparty.connection_id, expectedConnection), "failed to verify connection state");

64:   require(verifyClientState(host, connection, msg_.proofHeight, msg_.proofClient, msg_.clientStateBytes), "failed to verify clientState");

78:   require(found, "connection not found");

81:   revert("connection state is not INIT or TRYOPEN");

83:   revert("connection state is in INIT but the provided version is not supported");

85:   revert("connection state is in TRYOPEN but the provided version is not set in the previous connection versions");

88:   require(IBCClient.validateSelfClient(host, msg_.clientStateBytes), "failed to validate self client state");

104:  require(verifyConnectionState(host, connection, msg_.proofHeight, msg_.proofTry, msg_.counterpartyConnectionID, expectedConnection), "failed to verify connection state");

105:  require(verifyClientState(host, connection, msg_.proofHeight, msg_.proofClient, msg_.clientStateBytes), "failed to verify clientState");

120:  require(found, "connection not found");

122:  require(connection.state == ConnectionEnd.State.STATE_TRYOPEN, "connection state is not TRYOPEN");

138:  require(verifyConnectionState(host, connection, msg_.proofHeight, msg_.proofAck, connection.counterparty.connection_id, expectedConnection), "failed to verify connection state");
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/ibc/IBCConnection.sol

```solidity
File: contracts/ibc/IBCHost.sol

60:   require(address(clientRegistry[clientType]) == address(0), "clientImpl already exists");

72:   require(bytes(clientTypes[clientId]).length == 0, "clientId already exists");

73:   require(bytes(clientType).length > 0, "clientType must not be empty string");
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/ibc/IBCHost.sol

```solidity
File: contracts/mocks/MerkleTreeMock.sol

16:   require(vs.validators.length == total, "requested vs provided validator size differ");

18:   require(vs.validators[0].pub_key.ed25519.length > 0, "expected ed25519 public key, got empty array");
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/mocks/MerkleTreeMock.sol

```solidity
File: contracts/mocks/ProtoMock.sol

15:   require(
16:       keccak256(abi.encodePacked(chainID)) == keccak256(abi.encodePacked(header.signed_header.header.chain_id)),
17:       "invalid chain_id"
18:   );
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/mocks/ProtoMock.sol

```solidity
File: contracts/proto/Encoder.sol

70:   require(input.length < _MAX_UINT64, "Encoder: out of bounds (uint64)");
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/proto/Encoder.sol

```solidity
File: contracts/proto/ProtoBufRuntime.sol

891:  revert("not supported");
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/proto/ProtoBufRuntime.sol

```solidity
File: contracts/proto/TendermintHelper.sol

117:  require(h.header.validators_hash.length > 0, "Tendermint: hash can't be empty");

170:  require(sum <= maxTotalVotingPower, "total voting power should be guarded to not exceed");
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/proto/TendermintHelper.sol

```solidity
File: contracts/TendermintLightClient.sol

119:  require(ok, "LC: light block is invalid");

137:  require(ok, "LC: consensusState not found at trusted height");

140:  require(ok, "LC: client state is invalid");

172:  require(
173:      tmHeader.signed_header.header.height > tmHeader.trusted_height,
174:      "LC: header height consensus state height"
175:  );

201:  require(ok, "LC: failed to verify header");

402:  require(found, "LC: client state not found");

409:  require(found, "LC: consensus state not found");

415:  require(found, "LC: processed time not found");

421:  require(found, "LC: processed height not found");
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/TendermintLightClient.sol

```solidity
File: contracts/utils/Bytes.sol

6:    require(bz.length == 32, "Bytes: toBytes32 invalid size");

17:   require(_bytes.length >= _start + 8, "Bytes: toUint64 out of bounds");

24:   require(_bytes.length >= _start + 32, "Bytes: toUint256 out of bounds");
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/utils/Bytes.sol

```solidity
File: contracts/utils/crypto/Ed25519.sol

16:   require(signature.length == 64, "Ed25519: siganture length != 64");

17:   require(publicKey.length == 32, "Ed25519: pubkey length != 32");

27:   revert(0, "ed25519 precompile failed")
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/utils/crypto/Ed25519.sol

```solidity
File: contracts/utils/crypto/MerkleTree.sol

41:   require(input > 1, "MerkleTree: invalid input");
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/utils/crypto/MerkleTree.sol

```solidity
File: contracts/utils/crypto/Secp256k1.sol

71:   require(isCompressed(pubkey), "Secp256k1: PK must be compressed");
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/utils/crypto/Secp256k1.sol

```solidity
File: contracts/utils/Tendermint.sol

57:   require(untrustedHeader.header.height == trustedHeader.header.height + 1, "headers must be adjacent in height");

59:   require(!trustedHeader.isExpired(trustingPeriod, currentTime), "header can't be expired");

64:   require(
65:       untrustedHeader.header.validators_hash.toBytes32() == trustedHeader.header.next_validators_hash.toBytes32(),
66:       "expected old header next validators to match those from new header"
67:   );

91:   require(
92:       untrustedHeader.header.height != trustedHeader.header.height + 1,
93:       "LC: headers must be non adjacent in height"
94:   );

98:   require(
99:       trustedVals.hash() == trustedHeader.header.next_validators_hash.toBytes32(),
100:      "LC: headers trusted validators does not hash to latest trusted validators"
101:  );

103:  require(!trustedHeader.isExpired(trustingPeriod, currentTime), "header can't be expired");

130:  require(
131:      keccak256(abi.encodePacked(untrustedHeader.header.chain_id)) ==
132:          keccak256(abi.encodePacked(trustedHeader.header.chain_id)),
133:      "header belongs to another chain"
134:  );

135:  require(untrustedHeader.commit.height == untrustedHeader.header.height, "header and commit height mismatch");

138:  require(
139:      untrustedHeaderBlockHash == untrustedHeader.commit.block_id.hash.toBytes32(),
140:      "commit signs signs block failed"
141:  );

143:  require(
144:      untrustedHeader.header.height > trustedHeader.header.height,
145:      "expected new header height to be greater than one of old header"
146:  );

147:  require(
148:      untrustedHeader.header.time.gt(trustedHeader.header.time),
149:      "expected new header time to be after old header time"
150:  );

151:  require(
152:      Timestamp
153:          .Data({
154:              Seconds: int64(currentTime.Seconds) + int64(maxClockDrift.Seconds),
155:              nanos: int32(currentTime.nanos) + int32(maxClockDrift.nanos)
156:          })
157:          .gt(untrustedHeader.header.time),
158:      "new header has time from the future"
159:  );

162:  require(
163:      untrustedHeader.header.validators_hash.toBytes32() == validatorsHash,
164:      "expected new header validators to match those that were supplied at height XX"
165:  );

175:  require(trustLevel.denominator != 0, "trustLevel has zero Denominator");

197:  require(!seenVals[valIdx], "double vote of validator on the same commit");

232:  require(vals.validators.length == commit.signatures.length, "invalid commmit signatures");

234:  require(height == commit.height, "invalid commit height");

236:  require(commit.block_id.isEqual(blockID), "invalid commit -- wrong block ID");
```
https://github.com/ChorusOne/tendermint-sol/tree/main/contracts/utils/Tendermint.sol
