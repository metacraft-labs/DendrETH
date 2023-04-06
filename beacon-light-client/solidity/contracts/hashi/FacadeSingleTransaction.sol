// SPDX-License-Identifier: MIT
pragma solidity 0.8.9;

import '../bridge/src/truth/eth/BeaconLightClient.sol';
import '../bridge/src/struct/LightClientUpdateStruct.sol';

contract OracleAdapterFacade {
  mapping(uint256 => mapping(uint256 => bytes32)) public hashes;

  address oracleAddress;

  constructor(address _oracleAddress) {
    oracleAddress = _oracleAddress;
  }

  function updateHash(
    uint256 chainId,
    uint256 slot,
    LightClientUpdate memory update
  ) external {
    BeaconLightClient(oracleAddress).light_client_update(update);

    bytes32 oracle_hash = BeaconLightClient(oracleAddress)
      .optimistic_header_root();

    require(
      oracle_hash == update.attested_header_root,
      'Missmatch header hash'
    );

    hashes[chainId][slot] = update.attested_header_root;
  }
}
