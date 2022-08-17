# SNOWBRIDGE
## Gas Optimizations Report

### The usage of `++i` will cost less gas than `i++`. The same change can be applied to `i--` as well.
This change would save up to 6 gas per instance/loop.

_There are **15** instances of this issue:_

```solidity
File: ethereum/contracts/BasicInboundChannel.sol

46:   nonce++;

51:   for (uint256 i = 0; i < bundle.messages.length; i++) {
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/BasicInboundChannel.sol

```solidity
File: ethereum/contracts/BasicOutboundChannel.sol

40:   for (uint i = 0; i < defaultOperators.length; i++) {
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/BasicOutboundChannel.sol

```solidity
File: ethereum/contracts/BeefyClient.sol

382:  for (uint256 i = 0; i < signatureCount; i++) {
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/BeefyClient.sol

```solidity
File: ethereum/contracts/IncentivizedInboundChannel.sol

67:   nonce++;

73:   for (uint256 i = 0; i < bundle.messages.length; i++) {
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/IncentivizedInboundChannel.sol

```solidity
File: ethereum/contracts/IncentivizedOutboundChannel.sol

48:   for (uint i = 0; i < defaultOperators.length; i++) {
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/IncentivizedOutboundChannel.sol

```solidity
File: ethereum/contracts/utils/Bitfield.sol

49:   for (uint256 i = 0; found < n; i++) {

65:   found++;

81:   for (uint256 i = 0; i < bitsToSet.length; i++) {

95:   for (uint256 i = 0; i < self.length; i++) {
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/utils/Bitfield.sol

```solidity
File: ethereum/contracts/utils/MerkleProof.sol

46:   for (uint256 i = 0; i < proof.length; i++) {

67:   for (uint256 height = 0; width > 1; height++) {

94:   i++;
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/utils/MerkleProof.sol

```solidity
File: ethereum/contracts/utils/MMRProofVerification.sol

41:   for (uint256 currentPosition = 0; currentPosition < items.length; currentPosition++) {
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/utils/MMRProofVerification.sol

### State variables should be cached in stack variables rather than re-reading them.
The instances below point to the second+ access of a state variable within a function. Caching of a state variable replace each Gwarmaccess (100 gas) with a much cheaper stack read. Other less obvious fixes/optimizations include having local memory caches of state variable structs, or having local caches of state variable contracts/addresses.

_There are **10** instances of this issue:_

```solidity
File: ethereum/contracts/BasicOutboundChannel.sol

      /// @audit Cache `principal`. Used 2 times in `submit`
77:   if (principal != address(0x0000000000000000000000000000000000000042)) {
78:   require(_origin == principal, "Origin is not an authorized principal");
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/BasicOutboundChannel.sol

```solidity
File: ethereum/contracts/BeefyClient.sol

      /// @audit Cache `currentValidatorSet`. Used 2 times in `submitInitial`
196:  if (validatorSetID == currentValidatorSet.id) {
197:  vset = currentValidatorSet;

      /// @audit Cache `currentValidatorSet`. Used 2 times in `submitFinal`
247:  require(commitment.validatorSetID == currentValidatorSet.id);
249:  verifyCommitment(currentValidatorSet, request, commitment, proof);

      /// @audit Cache `nextValidatorSet`. Used 2 times in `submitInitial`
198:  } else if (validatorSetID == nextValidatorSet.id) {
199:  vset = nextValidatorSet;

      /// @audit Cache `nextValidatorSet`. Used 4 times in `submitFinal`
275:  require(commitment.validatorSetID == nextValidatorSet.id);
276:  require(leaf.nextAuthoritySetID == nextValidatorSet.id + 1);
278:  verifyCommitment(nextValidatorSet, request, commitment, proof);
289:  currentValidatorSet = nextValidatorSet;

      /// @audit Cache `nextRequestID`. Used 2 times in `submitInitial`
221:  requests[nextRequestID] = Request(
229:  emit NewRequest(nextRequestID, msg.sender);
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/BeefyClient.sol

```solidity
File: ethereum/contracts/DOTApp.sol

      /// @audit Cache `channels`. Used 2 times in `upgrade`
116:  Channel storage c1 = channels[ChannelId.Basic];
117:  Channel storage c2 = channels[ChannelId.Incentivized];
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/DOTApp.sol

```solidity
File: ethereum/contracts/ERC20App.sol

      /// @audit Cache `channels`. Used 2 times in `upgrade`
195:  Channel storage c1 = channels[ChannelId.Basic];
196:  Channel storage c2 = channels[ChannelId.Incentivized];
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/ERC20App.sol

```solidity
File: ethereum/contracts/ETHApp.sol

      /// @audit Cache `channels`. Used 2 times in `upgrade`
170:  Channel storage c1 = channels[ChannelId.Basic];
171:  Channel storage c2 = channels[ChannelId.Incentivized];
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/ETHApp.sol

```solidity
File: ethereum/contracts/IncentivizedOutboundChannel.sol

      /// @audit Cache `fee`. Used 2 times in `submit`
77:   feeController.handleFee(feePayer, fee);
79:   emit Message(msg.sender, nonce, fee, payload);
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/IncentivizedOutboundChannel.sol

### `internal` and `private` functions that are called only once should be inlined.
The execution of a non-inlined function would cost up to 40 more gas because of two extra `jump`s as well as some other instructions.

_There are **7** instances of this issue:_

```solidity
File: ethereum/contracts/DOTApp.sol

97:   function encodeCall(
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/DOTApp.sol

```solidity
File: ethereum/contracts/ERC20App.sol

142:  function encodeCall(

160:  function encodeCallWithParaId(

181:  function encodeCreateTokenCall(
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/ERC20App.sol

```solidity
File: ethereum/contracts/ETHApp.sol

119:  function encodeCall(

135:  function encodeCallWithParaId(
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/ETHApp.sol

```solidity
File: ethereum/contracts/MaliciousDOTApp.sol

92:   function encodeCall(
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/MaliciousDOTApp.sol

### Using `!= 0` on `uints` costs less gas than `> 0`.

_There are **2** instances of this issue:_

```solidity
File: ethereum/contracts/ERC20App.sol

130:  require(_amount > 0, "Must unlock a positive amount");
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/ERC20App.sol

```solidity
File: ethereum/contracts/ETHApp.sol

111:  require(_amount > 0, "Must unlock a positive amount");
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/ETHApp.sol

### It costs more gas to initialize non-`constant`/non-`immutable` variables to zero than to let the default of zero be applied
Not overwriting the default for stack variables saves 8 gas. Storage and memory variables have larger savings

_There are **14** instances of this issue:_

```solidity
File: ethereum/contracts/BasicInboundChannel.sol

51:   for (uint256 i = 0; i < bundle.messages.length; i++) {
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/BasicInboundChannel.sol

```solidity
File: ethereum/contracts/BasicOutboundChannel.sol

40:   for (uint i = 0; i < defaultOperators.length; i++) {
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/BasicOutboundChannel.sol

```solidity
File: ethereum/contracts/BeefyClient.sol

382:  for (uint256 i = 0; i < signatureCount; i++) {
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/BeefyClient.sol

```solidity
File: ethereum/contracts/IncentivizedInboundChannel.sol

73:   for (uint256 i = 0; i < bundle.messages.length; i++) {
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/IncentivizedInboundChannel.sol

```solidity
File: ethereum/contracts/IncentivizedOutboundChannel.sol

48:   for (uint i = 0; i < defaultOperators.length; i++) {
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/IncentivizedOutboundChannel.sol

```solidity
File: ethereum/contracts/utils/Bitfield.sol

47:   uint256 found = 0;

49:   for (uint256 i = 0; found < n; i++) {

81:   for (uint256 i = 0; i < bitsToSet.length; i++) {

94:   uint256 count = 0;

95:   for (uint256 i = 0; i < self.length; i++) {
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/utils/Bitfield.sol

```solidity
File: ethereum/contracts/utils/MerkleProof.sol

46:   for (uint256 i = 0; i < proof.length; i++) {

66:   uint256 i = 0;

67:   for (uint256 height = 0; width > 1; height++) {
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/utils/MerkleProof.sol

```solidity
File: ethereum/contracts/utils/MMRProofVerification.sol

41:   for (uint256 currentPosition = 0; currentPosition < items.length; currentPosition++) {
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/utils/MMRProofVerification.sol

### Using `private` rather than `public` for constants, saves gas
If needed, the values can be read from the verified contract source code, or if there are multiple values there can be a single getter function that returns a tuple of the values of all currently-public constants. Saves 3406-3606 gas in deployment gas due to the compiler not having to create non-payable getter functions for deployment calldata, not having to store the bytes of the value outside of where it's used, and not adding another entry to the method ID table

_There are **20** instances of this issue:_

```solidity
File: ethereum/contracts/BasicInboundChannel.sol

7:    uint256 public constant MAX_GAS_PER_MESSAGE = 100000;

8:    uint256 public constant GAS_BUFFER = 60000;
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/BasicInboundChannel.sol

```solidity
File: ethereum/contracts/BasicOutboundChannel.sol

12:   bytes32 public constant CONFIG_UPDATE_ROLE = keccak256("CONFIG_UPDATE_ROLE");
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/BasicOutboundChannel.sol

```solidity
File: ethereum/contracts/BeefyClient.sol

151:  uint256 public constant THRESHOLD_NUMERATOR = 3;

152:  uint256 public constant THRESHOLD_DENOMINATOR = 250;

153:  uint64 public constant BLOCK_WAIT_PERIOD = 3;
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/BeefyClient.sol

```solidity
File: ethereum/contracts/DOTApp.sol

21:   bytes32 public constant FEE_BURNER_ROLE = keccak256("FEE_BURNER_ROLE");

22:   bytes32 public constant INBOUND_CHANNEL_ROLE =

25:   bytes32 public constant CHANNEL_UPGRADE_ROLE =
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/DOTApp.sol

```solidity
File: ethereum/contracts/ERC20App.sol

58:   bytes32 public constant INBOUND_CHANNEL_ROLE =

61:   bytes32 public constant CHANNEL_UPGRADE_ROLE =
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/ERC20App.sol

```solidity
File: ethereum/contracts/ETHApp.sol

40:   bytes32 public constant REWARD_ROLE = keccak256("REWARD_ROLE");

47:   bytes32 public constant INBOUND_CHANNEL_ROLE =

50:   bytes32 public constant CHANNEL_UPGRADE_ROLE =
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/ETHApp.sol

```solidity
File: ethereum/contracts/IncentivizedInboundChannel.sol

27:   uint256 public constant MAX_GAS_PER_MESSAGE = 100000;

28:   uint256 public constant GAS_BUFFER = 60000;

31:   bytes32 public constant CONFIG_UPDATE_ROLE = keccak256("CONFIG_UPDATE_ROLE");
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/IncentivizedInboundChannel.sol

```solidity
File: ethereum/contracts/IncentivizedOutboundChannel.sol

14:   bytes32 public constant CONFIG_UPDATE_ROLE = keccak256("CONFIG_UPDATE_ROLE");
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/IncentivizedOutboundChannel.sol

```solidity
File: ethereum/contracts/MaliciousDOTApp.sol

27:   bytes32 public constant FEE_BURNER_ROLE = keccak256("FEE_BURNER_ROLE");

28:   bytes32 public constant INBOUND_CHANNEL_ROLE =
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/MaliciousDOTApp.sol

### Functions that are access-restricted from most users may be marked as `payable`
Marking a function as `payable` reduces gas cost since the compiler does not have to check whether a payment was provided or not. This change will save around 21 gas per function call.

_There are **20** instances of this issue:_

```solidity
File: ethereum/contracts/WrappedToken.sol

17:   function burn(address sender, uint256 amount, bytes memory data) external onlyOwner {

21:   function mint(address recipient, uint256 amount, bytes memory data) external onlyOwner {
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/WrappedToken.sol

```solidity
File: ethereum/contracts/BasicOutboundChannel.sol

31:   function initialize(
32:       address _configUpdater,
33:       address _principal,
34:       address[] memory defaultOperators
35:   )
36:   external onlyRole(DEFAULT_ADMIN_ROLE) {

49:   function authorizeDefaultOperator(address operator) external onlyRole(CONFIG_UPDATE_ROLE) {

54:   function revokeDefaultOperator(address operator) external onlyRole(CONFIG_UPDATE_ROLE) {

59:   function setPrincipal(address _principal) external onlyRole(CONFIG_UPDATE_ROLE) {
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/BasicOutboundChannel.sol

```solidity
File: ethereum/contracts/BeefyClient.sol

163:  function initialize(
164:      uint64 _initialBeefyBlock,
165:      ValidatorSet calldata _initialValidatorSet,
166:      ValidatorSet calldata _nextValidatorSet
167:  ) external onlyOwner {
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/BeefyClient.sol

```solidity
File: ethereum/contracts/DOTApp.sol

84:   function mint(
85:       bytes32 _sender,
86:       address _recipient,
87:       uint256 _amount
88:   ) external onlyRole(INBOUND_CHANNEL_ROLE) {

93:   function handleFee(address feePayer, uint256 _amount) external override onlyRole(FEE_BURNER_ROLE) {

112:  function upgrade(
113:      Channel memory _basic,
114:      Channel memory _incentivized
115:  ) external onlyRole(CHANNEL_UPGRADE_ROLE) {
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/DOTApp.sol

```solidity
File: ethereum/contracts/ERC20App.sol

124:  function unlock(
125:      address _token,
126:      bytes32 _sender,
127:      address _recipient,
128:      uint128 _amount
129:  ) public onlyRole(INBOUND_CHANNEL_ROLE) {

191:  function upgrade(
192:      Channel memory _basic,
193:      Channel memory _incentivized
194:  ) external onlyRole(CHANNEL_UPGRADE_ROLE) {
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/ERC20App.sol

```solidity
File: ethereum/contracts/ETHApp.sol

106:  function unlock(
107:      bytes32 _sender,
108:      address payable _recipient,
109:      uint128 _amount
110:  ) public onlyRole(INBOUND_CHANNEL_ROLE) {

155:  function handleReward(address payable _relayer, uint128 _amount)
156:      external
157:      override
158:      onlyRole(REWARD_ROLE)

166:  function upgrade(
167:      Channel memory _basic,
168:      Channel memory _incentivized
169:  ) external onlyRole(CHANNEL_UPGRADE_ROLE) {
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/ETHApp.sol

```solidity
File: ethereum/contracts/IncentivizedInboundChannel.sol

45:   function initialize(address _configUpdater, address _rewardController)
46:      external
47:      onlyRole(DEFAULT_ADMIN_ROLE)
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/IncentivizedInboundChannel.sol

```solidity
File: ethereum/contracts/IncentivizedOutboundChannel.sol

39:   function initialize(
40:      address _configUpdater,
41:      address _feeController,
42:      address[] memory defaultOperators
43:   )
44:   external onlyRole(DEFAULT_ADMIN_ROLE) {

53:   function setFee(uint256 _amount) external onlyRole(CONFIG_UPDATE_ROLE) {

63:   function authorizeDefaultOperator(address operator) external onlyRole(CONFIG_UPDATE_ROLE) {

68:   function revokeDefaultOperator(address operator) external onlyRole(CONFIG_UPDATE_ROLE) {
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/IncentivizedOutboundChannel.sol

### `++i`/`i++` should be `unchecked{++I}`/`unchecked{I++}` in `for`-loops
When an increment or any arithmetic operation is not possible to overflow it should be placed in `unchecked{}` block. \This is because of the default compiler overflow and underflow safety checks since Solidity version 0.8.0. \In for-loops it saves around 30-40 gas **per loop**

_There are **11** instances of this issue:_

```solidity
File: ethereum/contracts/BasicInboundChannel.sol

51:   for (uint256 i = 0; i < bundle.messages.length; i++) {
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/BasicInboundChannel.sol

```solidity
File: ethereum/contracts/BasicOutboundChannel.sol

40:   for (uint i = 0; i < defaultOperators.length; i++) {
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/BasicOutboundChannel.sol

```solidity
File: ethereum/contracts/BeefyClient.sol

382:  for (uint256 i = 0; i < signatureCount; i++) {
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/BeefyClient.sol

```solidity
File: ethereum/contracts/IncentivizedInboundChannel.sol

73:   for (uint256 i = 0; i < bundle.messages.length; i++) {
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/IncentivizedInboundChannel.sol

```solidity
File: ethereum/contracts/IncentivizedOutboundChannel.sol

48:   for (uint i = 0; i < defaultOperators.length; i++) {
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/IncentivizedOutboundChannel.sol

```solidity
File: ethereum/contracts/utils/Bitfield.sol

49:   for (uint256 i = 0; found < n; i++) {

81:   for (uint256 i = 0; i < bitsToSet.length; i++) {

95:   for (uint256 i = 0; i < self.length; i++) {
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/utils/Bitfield.sol

```solidity
File: ethereum/contracts/utils/MerkleProof.sol

46:   for (uint256 i = 0; i < proof.length; i++) {

67:   for (uint256 height = 0; width > 1; height++) {
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/utils/MerkleProof.sol

```solidity
File: ethereum/contracts/utils/MMRProofVerification.sol

41:   for (uint256 currentPosition = 0; currentPosition < items.length; currentPosition++) {
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/utils/MMRProofVerification.sol

### Splitting `require()` statements that use `&&` saves gas
Instead of using `&&` on single `require` check using two `require` checks can save gas

_There are **2** instances of this issue:_

```solidity
File: ethereum/contracts/BeefyClient.sol

374:  require(
375:      proof.signatures.length == signatureCount &&
376:          proof.indices.length == signatureCount &&
377:          proof.addrs.length == signatureCount &&
378:          proof.merkleProofs.length == signatureCount,
379:      "Validator proof is malformed"
380:  );
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/BeefyClient.sol

```solidity
File: ethereum/contracts/utils/Bits.sol

109:  require(0 < numBits && startIndex < 256 && startIndex + numBits <= 256);
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/utils/Bits.sol

### Use `calldata` instead of `memory` for function parameters
If a reference type function parameter is read-only, it is cheaper in gas to use calldata instead of memory. Calldata is a non-modifiable, non-persistent area where function arguments are stored, and behaves mostly like memory. Try to use calldata as a data location because it will avoid copies and also makes sure that the data cannot be modified.

_There are **24** instances of this issue:_

```solidity
File: ethereum/contracts/BasicOutboundChannel.sol

34:   address[] memory defaultOperators
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/BasicOutboundChannel.sol

```solidity
File: ethereum/contracts/BeefyClient.sol

329:  function minimumSignatureThreshold(ValidatorSet memory vset) internal pure returns (uint256) {

338:  ValidatorSet memory vset,

370:  ValidatorSet memory vset,

371:  uint256[] memory bitfield,

436:  ValidatorSet memory vset,

439:  bytes32[] memory proof
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/BeefyClient.sol

```solidity
File: ethereum/contracts/DOTApp.sol

113:  Channel memory _basic,

114:  Channel memory _incentivized
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/DOTApp.sol

```solidity
File: ethereum/contracts/ERC20App.sol

192:  Channel memory _basic,

193:  Channel memory _incentivized
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/ERC20App.sol

```solidity
File: ethereum/contracts/ETHApp.sol

167:  Channel memory _basic,

168:  Channel memory _incentivized
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/ETHApp.sol

```solidity
File: ethereum/contracts/IncentivizedOutboundChannel.sol

42:   address[] memory defaultOperators
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/IncentivizedOutboundChannel.sol

```solidity
File: ethereum/contracts/ParachainClient.sol

71:   bytes memory headPrefix,

72:   bytes memory headSuffix

83:   function createMMRLeaf(MMRLeafPartial memory leaf, bytes32 parachainHeadsRoot)
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/ParachainClient.sol

```solidity
File: ethereum/contracts/utils/Bitfield.sol

37:   uint256[] memory prior,

93:   function countSetBits(uint256[] memory self) public pure returns (uint256) {

111:  function isSet(uint256[] memory self, uint256 index)

121:  function set(uint256[] memory self, uint256 index) internal pure {

127:  function clear(uint256[] memory self, uint256 index) internal pure {
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/utils/Bitfield.sol

```solidity
File: ethereum/contracts/WrappedToken.sol

17:   function burn(address sender, uint256 amount, bytes memory data) external onlyOwner {

21:   function mint(address recipient, uint256 amount, bytes memory data) external onlyOwner {
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/WrappedToken.sol

### Replace `x <= y` with `x < y + 1`, and `x >= y` with `x > y - 1`
In the EVM, there is no opcode for `>=` or `<=`. When using greater than or equal, two operations are performed: `>` and `=`. Using strict comparison operators hence saves gas

_There are **8** instances of this issue:_

```solidity
File: ethereum/contracts/BasicInboundChannel.sol

43:   gasleft() >= (bundle.messages.length * MAX_GAS_PER_MESSAGE) + GAS_BUFFER,
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/BasicInboundChannel.sol

```solidity
File: ethereum/contracts/BeefyClient.sol

216:  bitfield.countSetBits() >= minimumSignatureThreshold(vset),

348:  block.number >= request.blockNumber + BLOCK_WAIT_PERIOD,

470:  require(block.number >= request.blockNumber + BLOCK_WAIT_PERIOD, "wait period not over");
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/BeefyClient.sol

```solidity
File: ethereum/contracts/ERC20App.sol

132:  _amount <= balances[_token],
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/ERC20App.sol

```solidity
File: ethereum/contracts/IncentivizedInboundChannel.sol

64:   gasleft() >= (bundle.messages.length * MAX_GAS_PER_MESSAGE) + GAS_BUFFER,
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/IncentivizedInboundChannel.sol

```solidity
File: ethereum/contracts/utils/Bitfield.sol

42:   n <= countSetBits(prior),
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/utils/Bitfield.sol

```solidity
File: ethereum/contracts/utils/Bits.sol

109:  require(0 < numBits && startIndex < 256 && startIndex + numBits <= 256);
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/utils/Bits.sol

### Use `immutable` & `constant` for state variables that do not change their value

_There are **4** instances of this issue:_

```solidity
File: ethereum/contracts/BasicInboundChannel.sol

14:   ParachainClient public parachainClient;
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/BasicInboundChannel.sol

```solidity
File: ethereum/contracts/DOTApp.sol

19:   WrappedToken public token;
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/DOTApp.sol

```solidity
File: ethereum/contracts/IncentivizedInboundChannel.sol

35:   ParachainClient public parachainClient;
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/IncentivizedInboundChannel.sol

```solidity
File: ethereum/contracts/MaliciousDOTApp.sol

25:   WrappedToken public token;
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/MaliciousDOTApp.sol

### `require()/revert()` strings longer than 32 bytes cost extra gas
Each extra memory word of bytes past the original 32 incurs an MSTORE which costs 3 gas

_There are **9** instances of this issue:_

```solidity
File: ethereum/contracts/BasicInboundChannel.sol

42:   require(
43:       gasleft() >= (bundle.messages.length * MAX_GAS_PER_MESSAGE) + GAS_BUFFER,
44:       "insufficient gas for delivery of all messages"
45:   );
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/BasicInboundChannel.sol

```solidity
File: ethereum/contracts/BasicOutboundChannel.sol

78:   require(_origin == principal, "Origin is not an authorized principal");
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/BasicOutboundChannel.sol

```solidity
File: ethereum/contracts/BeefyClient.sol

353:  require(commitment.blockNumber > latestBeefyBlock, "Commitment blocknumber is too old");
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/BeefyClient.sol

```solidity
File: ethereum/contracts/ERC20App.sol

118:  require(
119:      IERC20(_token).transferFrom(msg.sender, address(this), _amount),
120:      "Contract token allowances insufficient to complete this lock request"
121:  );

131:  require(
132:      _amount <= balances[_token],
133:      "ERC20 token balances insufficient to fulfill the unlock request"
134:  );
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/ERC20App.sol

```solidity
File: ethereum/contracts/ETHApp.sol

81:   require(msg.value > 0, "Value of transaction must be positive");
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/ETHApp.sol

```solidity
File: ethereum/contracts/IncentivizedInboundChannel.sol

63:   require(
64:       gasleft() >= (bundle.messages.length * MAX_GAS_PER_MESSAGE) + GAS_BUFFER,
65:       "insufficient gas for delivery of all messages"
66:   );
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/IncentivizedInboundChannel.sol

```solidity
File: ethereum/contracts/IncentivizedOutboundChannel.sol

76:   require(isOperatorFor(msg.sender, feePayer), "Caller is not an operator for fee payer");
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/IncentivizedOutboundChannel.sol

```solidity
File: ethereum/contracts/utils/Bitfield.sol

41:   require(
42:       n <= countSetBits(prior),
43:       "`n` must be <= number of set bits in `prior`"
44:   );
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/utils/Bitfield.sol

### Use custom errors rather than `revert()`/`require()` strings to save gas
Custom errors are available from solidity version 0.8.4. Custom errors save ~50 gas each time they're hitby avoiding having to allocate and store the revert string. Not defining the strings also save deployment gas

_There are **42** instances of this issue:_

```solidity
File: ethereum/contracts/BasicInboundChannel.sol

39:   require(parachainClient.verifyCommitment(commitment, proof), "Invalid proof");

40:   require(bundle.sourceChannelID == sourceChannelID, "Invalid source channel");

41:   require(bundle.nonce == nonce + 1, "Invalid nonce");

42:   require(
43:       gasleft() >= (bundle.messages.length * MAX_GAS_PER_MESSAGE) + GAS_BUFFER,
44:       "insufficient gas for delivery of all messages"
45:   );
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/BasicInboundChannel.sol

```solidity
File: ethereum/contracts/BasicOutboundChannel.sol

76:   require(isOperatorFor(msg.sender, _origin), "Caller is unauthorized");

78:   require(_origin == principal, "Origin is not an authorized principal");
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/BasicOutboundChannel.sol

```solidity
File: ethereum/contracts/BeefyClient.sol

201:  revert("Unknown validator set");

205:  require(
206:      isValidatorInSet(vset, proof.addr, proof.index, proof.merkleProof),
207:      "invalid validator proof"
208:  );

212:  require(ECDSA.recover(commitmentHash, proof.signature) == proof.addr, "Invalid signature");

215:  require(
216:      bitfield.countSetBits() >= minimumSignatureThreshold(vset),
217:      "Not enough claims"
218:  );

280:  require(
281:      MMRProofVerification.verifyLeafProof(
282:          commitment.payload.mmrRootHash,
283:          keccak256(encodeMMRLeaf(leaf)),
284:          leafProof
285:      ),
286:      "Invalid leaf proof"
287:  );

344:  require(msg.sender == request.sender, "Sender address invalid");

347:  require(
348:      block.number >= request.blockNumber + BLOCK_WAIT_PERIOD,
349:      "Block wait period not over"
350:  );

353:  require(commitment.blockNumber > latestBeefyBlock, "Commitment blocknumber is too old");

374:  require(
375:      proof.signatures.length == signatureCount &&
376:          proof.indices.length == signatureCount &&
377:          proof.addrs.length == signatureCount &&
378:          proof.merkleProofs.length == signatureCount,
379:      "Validator proof is malformed"
380:  );

391:  require(bitfield.isSet(index), "Validator not in bitfield");

397:  require(isValidatorInSet(vset, addr, index, merkleProof), "invalid validator proof");

400:  require(ECDSA.recover(commitmentHash, signature) == addr, "Invalid signature");

470:  require(block.number >= request.blockNumber + BLOCK_WAIT_PERIOD, "wait period not over");
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/BeefyClient.sol

```solidity
File: ethereum/contracts/ChannelAccess.sol

38:   require(msg.sender != operator, "Revoking self as operator");

46:   require(msg.sender != operator, "Authorizing self as operator");

54:   require(msg.sender != operator, "Revoking self as operator");
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/ChannelAccess.sol

```solidity
File: ethereum/contracts/DOTApp.sol

70:   require(
71:       _channelId == ChannelId.Basic ||
72:           _channelId == ChannelId.Incentivized,
73:       "Invalid channel ID"
74:   );
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/DOTApp.sol

```solidity
File: ethereum/contracts/ERC20App.sol

89:   require(
90:       _channelId == ChannelId.Basic ||
91:           _channelId == ChannelId.Incentivized,
92:       "Invalid channel ID"
93:   );

118:  require(
119:      IERC20(_token).transferFrom(msg.sender, address(this), _amount),
120:      "Contract token allowances insufficient to complete this lock request"
121:  );

130:  require(_amount > 0, "Must unlock a positive amount");

131:  require(
132:      _amount <= balances[_token],
133:      "ERC20 token balances insufficient to fulfill the unlock request"
134:  );
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/ERC20App.sol

```solidity
File: ethereum/contracts/ETHApp.sol

81:   require(msg.value > 0, "Value of transaction must be positive");

82:   require(
83:       _channelId == ChannelId.Basic ||
84:           _channelId == ChannelId.Incentivized,
85:       "Invalid channel ID"
86:   );

111:  require(_amount > 0, "Must unlock a positive amount");

114:  require(success, "Unable to send Ether");
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/ETHApp.sol

```solidity
File: ethereum/contracts/IncentivizedInboundChannel.sol

60:   require(parachainClient.verifyCommitment(commitment, proof), "Invalid proof");

61:   require(bundle.sourceChannelID == sourceChannelID, "Invalid source channel");

62:   require(bundle.nonce == nonce + 1, "Invalid nonce");

63:   require(
64:       gasleft() >= (bundle.messages.length * MAX_GAS_PER_MESSAGE) + GAS_BUFFER,
65:       "insufficient gas for delivery of all messages"
66:   );
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/IncentivizedInboundChannel.sol

```solidity
File: ethereum/contracts/IncentivizedOutboundChannel.sol

76:   require(isOperatorFor(msg.sender, feePayer), "Caller is not an operator for fee payer");
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/IncentivizedOutboundChannel.sol

```solidity
File: ethereum/contracts/MaliciousDOTApp.sol

64:   require(
65:       _channelId == ChannelId.Basic ||
66:           _channelId == ChannelId.Incentivized,
67:       "Invalid channel ID"
68:   );
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/MaliciousDOTApp.sol

```solidity
File: ethereum/contracts/utils/Bitfield.sol

41:   require(
42:       n <= countSetBits(prior),
43:       "`n` must be <= number of set bits in `prior`"
44:   );
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/utils/Bitfield.sol

```solidity
File: ethereum/contracts/utils/MerkleProof.sol

64:   require(pos < width, "Merkle position is too high");

78:   require(i < proof.length, "Merkle proof is too short");
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/utils/MerkleProof.sol

```solidity
File: ethereum/contracts/WrappedToken.sol

29:   revert("not-supported");

33:   revert("not-supported");
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/WrappedToken.sol

### Expressions for constant values such as a call to `keccak256()`, should use `immutable` rather than `constant`
It is expected that the value should be converted into a constant value at compile time. But actually the expression is re-calculated each time the constant is referenced.

_There are **6** instances of this issue:_

```solidity
File: ethereum/contracts/BasicOutboundChannel.sol

12:   bytes32 public constant CONFIG_UPDATE_ROLE = keccak256("CONFIG_UPDATE_ROLE");
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/BasicOutboundChannel.sol

```solidity
File: ethereum/contracts/DOTApp.sol

21:   bytes32 public constant FEE_BURNER_ROLE = keccak256("FEE_BURNER_ROLE");
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/DOTApp.sol

```solidity
File: ethereum/contracts/ETHApp.sol

40:   bytes32 public constant REWARD_ROLE = keccak256("REWARD_ROLE");
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/ETHApp.sol

```solidity
File: ethereum/contracts/IncentivizedInboundChannel.sol

31:   bytes32 public constant CONFIG_UPDATE_ROLE = keccak256("CONFIG_UPDATE_ROLE");
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/IncentivizedInboundChannel.sol

```solidity
File: ethereum/contracts/IncentivizedOutboundChannel.sol

14:   bytes32 public constant CONFIG_UPDATE_ROLE = keccak256("CONFIG_UPDATE_ROLE");
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/IncentivizedOutboundChannel.sol

```solidity
File: ethereum/contracts/MaliciousDOTApp.sol

27:   bytes32 public constant FEE_BURNER_ROLE = keccak256("FEE_BURNER_ROLE");
```
https://github.com/Snowfork/snowbridge/tree/main/ethereum/contracts/MaliciousDOTApp.sol

### Use a more recent version of solidity
- Use a solidity version of at least 0.8.2 to get simple compiler automatic inlining \
- Use a solidity version of at least 0.8.3 to get cheaper multiple storage reads \
- Use a solidity version of at least 0.8.4 to get custom errors, which are cheaper at deployment than revert()/require() strings \
- Use a solidity version of at least 0.8.10 to have external calls skip contract existence checks if the external call has a return value
