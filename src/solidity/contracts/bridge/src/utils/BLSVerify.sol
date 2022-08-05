pragma solidity 0.8.9;
import "./signature.sol";
import "hardhat/console.sol";

contract BLSVerify is Verifier {
    bytes4 constant private DOMAIN_SYNC_COMMITTEE = 0x07000000;
    bytes32 constant private GENESIS_VALIDATORS_ROOT = 0x4b363db94e286120d76eb905340fdd4e54bfe9f06bf33ff6cf5ad27f511bfe95;
    // Fp is a field element with the high-order part stored in `a`.
    struct Fp {
        uint a;
        uint b;
    }

    // Fp2 is an extension field element with the coefficient of the
    // quadratic non-residue stored in `b`, i.e. p = a + i * b
    struct Fp2 {
        Fp a;
        Fp b;
    }

    function FpToArray55_7(Fp memory fp) public pure returns (uint256[7] memory) {
        uint256[7] memory result;
        uint256 mask = ((1 << 55) - 1);
        result[0] = (fp.b & (mask << (55 * 0))) >> 55 * 0;
        result[1] = (fp.b & (mask << (55 * 1))) >> 55 * 1;
        result[2] = (fp.b & (mask << (55 * 2))) >> 55 * 2;
        result[3] = (fp.b & (mask << (55 * 3))) >> 55 * 3;
        result[4] = (fp.b & (mask << (55 * 4))) >> 55 * 4;
        uint256 newMask = (1 << 20) - 1;
        result[4] = result[4] | (fp.a & newMask) << 36;
        result[5] = (fp.a & (mask << 19)) >> 19;
        result[6] = (fp.a & (mask << (55+19))) >> (55+19);

        return result;
    }

    // function compute_domain(bytes4 domain_type, bytes4 fork_version, bytes32 genesis_validators_root) internal pure returns (bytes32){
    //     bytes32 fork_data_root = compute_fork_data_root(fork_version, genesis_validators_root);
    //     return bytes32(domain_type) | fork_data_root >> 32;
    // }

    // function compute_signing_root(BeaconBlockHeader memory beacon_header, bytes32 domain) internal pure returns (bytes32){
    //     return hash_tree_root(SigningData({
    //             object_root: hash_tree_root(beacon_header),
    //             domain: domain
    //         })
    //     );
    // }

    // function hash_tree_root(SigningData memory signing_data) internal pure returns (bytes32) {
    //     return hash_node(signing_data.object_root, signing_data.domain);
    // }

    struct SigningData {
        bytes32 object_root;
        bytes32 domain;
    }

    // function hashToField(bytes32 message) public view returns (Fp2[2] memory result) {
    //     bytes memory some_bytes = expandMessage(message);
    //     result[0] = Fp2(
    //         convertSliceToFp(some_bytes, 0, 64),
    //         convertSliceToFp(some_bytes, 64, 128)
    //     );
    //     result[1] = Fp2(
    //         convertSliceToFp(some_bytes, 128, 192),
    //         convertSliceToFp(some_bytes, 192, 256)
    //     );
    // }

    // function verify1(
    //     bytes memory pubKey,
    //     Fp memory pubKeyY,
    //     bytes memory signatureKey,
    //     Fp2 memory signatureY
    // ) {

    //     bytes32 domain = compute_domain(DOMAIN_SYNC_COMMITTEE, 0x00000000, GENESIS_VALIDATORS_ROOT);
    //     bytes32 signing_root = convertSliceToFpcompute_signing_root(header, domain);
    //     Fp2[2] memory message_on_field = hashToField(message);


    // }

    function verify(
        uint[2] memory a,
        uint[2][2] memory b,
        uint[2] memory c,
        uint[70] memory input) public view returns(bool) {
            for(uint256 i = 0; i < input.length; i++) {
                console.log(input[i]);
            }

            return verifyProof(a, b, c, input);
        }
}
