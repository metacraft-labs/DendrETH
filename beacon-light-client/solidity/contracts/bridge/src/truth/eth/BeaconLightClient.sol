// SPDX-License-Identifier: MIT
pragma solidity 0.8.9;

import '../../utils/LightClientUpdateVerifier.sol';

contract BeaconLightClient is LightClientUpdateVerifier {
  struct LightClientUpdate {
    bytes32 attested_header_root;
    bytes32 finalized_header_root;
    bytes32 finalized_execution_state_root;
    uint256[2] a;
    uint256[2][2] b;
    uint256[2] c;
  }

  bytes32 _optimistic_header_root;

  bytes32 _finalized_header_root;

  bytes32 _finalized_execution_state_root;

  constructor(
    bytes32 __optimistic_header_root,
    bytes32 __finalized_header_root,
    bytes32 __execution_state_root
  ) {
    _optimistic_header_root = __optimistic_header_root;
    _finalized_header_root = __finalized_header_root;
    _finalized_execution_state_root = __execution_state_root;
  }

  function execution_state_root() public view returns (bytes32) {
    return _finalized_execution_state_root;
  }

  function optimistic_header_root() public view returns (bytes32) {
    return _optimistic_header_root;
  }

  function finalized_header_root() public view returns (bytes32) {
    return _finalized_header_root;
  }

  function light_client_update(LightClientUpdate calldata update)
    external
    payable
  {
    require(
      verifyUpdate(
        update.a,
        update.b,
        update.c,
        optimistic_header_root(),
        update.attested_header_root,
        update.finalized_header_root,
        update.finalized_execution_state_root
      ),
      '!proof'
    );

    _optimistic_header_root = update.attested_header_root;
    _finalized_header_root = update.finalized_header_root;
    _finalized_execution_state_root = update.finalized_execution_state_root;
  }
}
