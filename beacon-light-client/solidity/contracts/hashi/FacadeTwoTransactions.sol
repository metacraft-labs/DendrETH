// SPDX-License-Identifier: MIT
pragma solidity 0.8.9;

import '../bridge/src/truth/eth/BeaconLightClient.sol';

contract OracleAdapterFacade {
  mapping(uint256 => mapping(uint256 => bytes32)) public hashes;

  address oracleAddress;

  constructor(address _oracleAddress) {
    oracleAddress = _oracleAddress;
  }

  function updateHash(
    uint256 chainId,
    uint256 slot,
    bytes32 header_hash
  ) external {
    bytes32 oracle_hash = BeaconLightClient(oracleAddress)
      .optimistic_header_root();

    require(oracle_hash == header_hash, 'Missmatch header hash');

    hashes[chainId][slot] = header_hash;
  }
}
