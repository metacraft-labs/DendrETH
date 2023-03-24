// SPDX-License-Identifier: MIT
pragma solidity ^0.8.17;

import '../../hashi/contracts/adapters/BlockHashOracleAdapter.sol';
import '../bridge/src/truth/eth/BeaconLightClient.sol';

contract OracleAdapterFacade is BlockHashOracleAdapter {
  address oracleAddress;

  constructor(address _oracleAddress) {
    oracleAddress = _oracleAddress;
  }

  function updateHash(
    uint256 chainId,
    uint256 slot,
    bytes32 header_hash
  ) external {
    uint256 oracle_slot = BeaconLightClient(oracleAddress)
      .optimistic_header_slot();

    require(oracle_slot == slot, 'Missmatch header slot');

    bytes32 oracle_hash = BeaconLightClient(oracleAddress)
      .optimistic_header_root();

    require(oracle_hash == header_hash, 'Missmatch header hash');

    hashes[chainId][slot] = header_hash;
  }
}
