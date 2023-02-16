// SPDX-License-Identifier: MIT
pragma solidity 0.8.9;

import '../../utils/LightClientUpdateVerifier.sol';
import '../../spec/BeaconChain.sol';

contract BeaconLightClient is LightClientUpdateVerifier {
  struct LightClientUpdate {
    bytes32 attested_header_root;

    bytes32 finalized_header_root;

    bytes32 finalized_execution_state_root;

    uint256[2] a;
    uint256[2][2] b;
    uint256[2] c;
  }

  bytes32 optimistic_header_root;

  bytes32 finalized_header_root;

  bytes32 finalized_execution_state_root;

  constructor(
    bytes32 _finalized_header_root
    bytes32 _execution_state_root
  ) {
    optimistic_header_root = _finalized_header_root;
    finalized_header_root = _finalized_header_root;
    finalized_execution_state_root = _execution_state_root;
  }

  function state_root() public view returns (bytes32) {
    return finalized_header.state_root;
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
        finalized_header_root,
        update.attested_header_root,
        update.finalized_header_root,
        update.finalized_execution_state_root,
      ),
      '!proof'
    );

    // Maybe we should also validate if header.slot > finalized_header.slot

    optimistic_header_root = update.attested_header_root;
    finalized_header_root = update.finalized_header;
    finalized_execution_state_root = update.finalized_execution_state_root;
  }
}
