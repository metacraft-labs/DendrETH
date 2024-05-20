use std::{marker::PhantomData, str::FromStr};

use num::BigUint;
use plonky2::{
    field::goldilocks_field::GoldilocksField,
    hash::hash_types::HashOutTarget,
    iop::{
        target::{BoolTarget, Target},
        witness::{PartialWitness, WitnessWrite},
    },
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData, CommonCircuitData, VerifierOnlyCircuitData},
        config::PoseidonGoldilocksConfig,
        proof::ProofWithPublicInputsTarget,
    },
};
use plonky2_circuit_serializer::serializer::{CustomGateSerializer, CustomGeneratorSerializer};
use plonky2_u32::gadgets::arithmetic_u32::U32Target;
use serde_json::json;

use crate::{
    biguint::{BigUintTarget, CircuitBuilderBiguint},  is_active_validator::get_validator_status, is_valid_merkle_branch::is_valid_merkle_branch_sha256, is_valid_merkle_branch_poseidon::{
        is_valid_merkle_branch_poseidon, is_valid_merkle_branch_poseidon_result,
    }, sha256::sha256_pair, utils::{if_biguint, ssz_num_from_bits}, validator_hash_tree_root_poseidon::hash_tree_root_validator_poseidon,
};

use super::{deposit_hash_tree_root_poseidon::hash_tree_root_deposit_poseidon, objects::{AccumulatedDataTargets, DepositAccumulatorLeafTargets, NodeTargets, RangeObject, ValidatorStatsTargets}, utils::set_node_public_variables};

pub fn deposit_accumulator_leaf_circuit(
    builder: &mut CircuitBuilder<GoldilocksField, 2>,
    bls_common_data: &CommonCircuitData<GoldilocksField, 2>,
    bls_verifier_data: &VerifierOnlyCircuitData<PoseidonGoldilocksConfig, 2>,
) -> DepositAccumulatorLeafTargets {
    let deposit_hash_tree_root = hash_tree_root_deposit_poseidon(builder);
    let is_valid_merkle_tree_deposit_branch = is_valid_merkle_branch_poseidon(builder, 32);

    builder.connect_hashes(
        is_valid_merkle_tree_deposit_branch.leaf,
        deposit_hash_tree_root.hash_tree_root,
    );

    let _true = builder._true();
    let _false = builder._false();

    let domain = [
        _false, _false, _false, _false, _false, _false, _true, _true, _false, _false, _false,
        _false, _false, _false, _false, _false, _false, _false, _false, _false, _false, _false,
        _false, _false, _false, _false, _false, _false, _false, _false, _false, _false, _true,
        _true, _true, _true, _false, _true, _false, _true, _true, _false, _true, _false, _false,
        _true, _false, _true, _true, _true, _true, _true, _true, _true, _false, _true, _false,
        _true, _false, _false, _false, _false, _true, _false, _true, _true, _false, _true, _false,
        _false, _false, _true, _false, _true, _true, _false, _true, _false, _true, _false, _false,
        _false, _true, _false, _false, _false, _false, _false, _false, _false, _true, _true,
        _false, _false, _false, _false, _false, _false, _true, _false, _false, _true, _true, _true,
        _true, _false, _false, _true, _true, _false, _false, _false, _true, _true, _true, _false,
        _true, _true, _true, _true, _false, _true, _true, _false, _true, _true, _true, _false,
        _true, _true, _false, _true, _false, _false, _true, _true, _false, _false, _false, _false,
        _true, _false, _false, _true, _true, _false, _false, _true, _false, _true, _true, _true,
        _true, _false, _false, _true, _true, _false, _true, _true, _false, _true, _false, _false,
        _false, _false, _true, _true, _false, _false, _false, _false, _false, _false, _false,
        _false, _false, _false, _true, _true, _true, _true, _false, _true, _false, _false, _true,
        _false, _false, _false, _true, _true, _false, _false, _true, _false, _false, _false,
        _false, _false, _true, _true, _false, _true, _true, _false, _false, _true, _true, _true,
        _true, _true, _false, _false, _false, _false, _true, _true, _true, _false, _true, _false,
        _false, _false, _true, _true, _true, _false, _true, _false, _true, _false, _true, _false,
        _false, _true, _true, _false, _false, _false, _false, _false, _true, _true, _false, _false,
        _false, _true, _true, _false, _true, _false, _true, _false, _false, _true,
    ];

    let message = sha256_pair(
        builder,
        &domain,
        &deposit_hash_tree_root
            .deposit
            .deposit_message_hash_tree_root,
    );
    let bls_signature_proof = builder.add_virtual_proof_with_pis(bls_common_data);

    verify_bls_signature(
        builder,
        &deposit_hash_tree_root.deposit.pubkey,
        &deposit_hash_tree_root.deposit.signature,
        &message,
        &bls_signature_proof,
        bls_common_data,
        bls_verifier_data,
    );

    let eth1_deposit_index = builder.add_virtual_biguint_target(2);
    let deposit_is_processed = builder.cmp_biguint(
        &deposit_hash_tree_root.deposit.deposit_index,
        &eth1_deposit_index,
    );

    let signature_is_valid =
        BoolTarget::new_unsafe(*bls_signature_proof.public_inputs.last().unwrap());
    let validator_is_definitely_on_chain: BoolTarget =
        builder.and(deposit_is_processed, signature_is_valid);

    let is_valid_commitment_mapper_proof = is_valid_merkle_branch_poseidon_result(builder, 40);
    let validator_hash_tree_root = hash_tree_root_validator_poseidon(builder);

    builder.connect(
        is_valid_commitment_mapper_proof.is_valid.target,
        validator_is_definitely_on_chain.target,
    );

    // connect that validators are the same
    let is_dummy = builder.add_virtual_bool_target_safe();
    let one = builder.one();
    let not_is_dummy = builder.not(is_dummy);

    for i in 0..384 {
        builder.connect(
            validator_hash_tree_root.validator.pubkey[i].target,
            deposit_hash_tree_root.deposit.pubkey[i].target,
        );

        // connect if is dummy pubkey is max
        let is_one = builder.is_equal(validator_hash_tree_root.validator.pubkey[i].target, one);
        let should_be_true = builder.or(not_is_dummy, is_one);
        builder.connect(one, should_be_true.target);
    }

    builder.connect_hashes(
        validator_hash_tree_root.hash_tree_root,
        is_valid_commitment_mapper_proof.leaf,
    );

    let is_valid_merkle_branch_balances = is_valid_merkle_branch_sha256(builder, 22);
    let four = builder.constant_biguint(&BigUint::from_str("4").unwrap());
    let validator_index_big_uint = BigUintTarget {
        limbs: vec![U32Target(is_valid_commitment_mapper_proof.index)],
    };

    let balance_index_big_uint = builder.div_biguint(&validator_index_big_uint, &four);
    let balance_index_target = balance_index_big_uint.limbs[0].0;
    builder.connect(
        balance_index_target,
        is_valid_merkle_tree_deposit_branch.index,
    );

    let current_epoch = builder.add_virtual_biguint_target(2);

    // TODO: Should work with inner index
    let balance_inner_index = builder.rem_biguint(&validator_index_big_uint, &four);
    let balance_inner_index = 0;
    let balance = ssz_num_from_bits(
        builder,
        &is_valid_merkle_branch_balances.leaf
            [((balance_inner_index % 4) * 64)..(((balance_inner_index % 4) * 64) + 64)],
    );

    let zero = builder.zero_biguint();

    let (is_non_activated_validator, is_valid_validator, is_exited_validator) =
        get_validator_status(
            builder,
            &validator_hash_tree_root.validator.activation_epoch,
            &current_epoch,
            &validator_hash_tree_root.validator.exit_epoch,
        );

    let will_be_counted = builder.and(validator_is_definitely_on_chain, is_valid_validator);

    // TODO: should be if validator is relevant: A validator is relevant for the total locked value computation only if it is included in the validators accumulator and its activationEpoch and withdrawableEpoch enclose the currentEpoch.
    let balance = if_biguint(builder, will_be_counted, &balance, &zero);

    let active_count = will_be_counted.target;

    let non_activated_count =
        builder.and(validator_is_definitely_on_chain, is_non_activated_validator);

    let exited_count = builder.and(validator_is_definitely_on_chain, is_exited_validator);

    let slashed_count = builder.and(
        validator_is_definitely_on_chain,
        validator_hash_tree_root.validator.slashed,
    );

    let leftmost = RangeObject {
        pubkey: validator_hash_tree_root.validator.pubkey.clone(),
        deposit_index: deposit_hash_tree_root.deposit.deposit_index.clone(),
        is_counted: validator_is_definitely_on_chain,
        is_dummy: is_dummy,
    };

    let rightmost = RangeObject {
        pubkey: validator_hash_tree_root.validator.pubkey,
        deposit_index: deposit_hash_tree_root.deposit.deposit_index.clone(),
        is_counted: validator_is_definitely_on_chain,
        is_dummy: is_dummy,
    };

    let accumulated = AccumulatedDataTargets {
            balance_sum: balance,
            deposits_count: one,
            validator_stats: ValidatorStatsTargets {
                non_activated_validators_count: non_activated_count.target,
                active_validators_count: active_count,
                exited_validators_count: exited_count.target,
                slashed_validators_count: slashed_count.target,    
            }, 
    };

    let node = NodeTargets {
        leftmost: leftmost,
        rightmost: rightmost,
        accumulated: accumulated,
        current_epoch: current_epoch,
        eth1_deposit_index: eth1_deposit_index,
        commitment_mapper_proof_root: is_valid_commitment_mapper_proof.root,
        merkle_tree_deposit_branch_root: is_valid_merkle_tree_deposit_branch.root,
    };

    set_node_public_variables(builder, &node);

    DepositAccumulatorLeafTargets {
        validator: validator_hash_tree_root.validator.clone(),
        validator_deposit: deposit_hash_tree_root.deposit.clone(),
        commitment_mapper_proof: is_valid_commitment_mapper_proof.branch,
        validator_index: is_valid_commitment_mapper_proof.index,
        validator_deposit_proof: is_valid_merkle_tree_deposit_branch.branch,
        validator_deposit_index: is_valid_merkle_tree_deposit_branch.index,
        balance_tree_root: is_valid_merkle_branch_balances.root,
        balance_leaf: is_valid_merkle_branch_balances.leaf,
        balance_proof: is_valid_merkle_branch_balances.branch.try_into().unwrap(),
        bls_signature_proof: bls_signature_proof,
        is_dummy: is_dummy,
        node: node
    }
}

fn verify_bls_signature(
    builder: &mut CircuitBuilder<GoldilocksField, 2>,
    pubkey: &[BoolTarget; 384],
    signature: &[BoolTarget; 768],
    message: &[BoolTarget; 256],
    bls_signature_proof: &ProofWithPublicInputsTarget<2>,
    bls_common_data: &CommonCircuitData<GoldilocksField, 2>,
    bls_verifier_data: &VerifierOnlyCircuitData<PoseidonGoldilocksConfig, 2>,
) {
    let bls_verifier_circuit_targets = builder.constant_verifier_data(bls_verifier_data);

    builder.verify_proof::<PoseidonGoldilocksConfig>(
        &bls_signature_proof,
        &bls_verifier_circuit_targets,
        bls_common_data,
    );

    for i in (0..384).step_by(8) {
        let byte = builder.le_sum(pubkey[i..(i + 8)].iter().copied());
        builder.connect(byte, bls_signature_proof.public_inputs[i])
    }

    for i in (0..768).step_by(8) {
        let byte = builder.le_sum(signature[i..(i + 8)].iter().copied());
        builder.connect(byte, bls_signature_proof.public_inputs[i + 384]);
    }

    for i in (0..256).step_by(8) {
        let byte = builder.le_sum(message[i..(i + 8)].iter().copied());
        builder.connect(byte, bls_signature_proof.public_inputs[i + 384 + 768]);
    }
}

#[test]
pub fn test_deposit_accumulator_leaf_circuit() {
    let config = CircuitConfig::standard_recursion_config();
    let mut builder = CircuitBuilder::<GoldilocksField, 2>::new(config);

    // let bls_common_data = plonky2::circuit_data::CommonCircuitData::<GoldilocksField, 2>::new();
    // let bls_verifier_data =
    //     VerifierOnlyCircuitData::<PoseidonGoldilocksConfig, 2>;

    let bls_data = CircuitData::<GoldilocksField, PoseidonGoldilocksConfig, 2>::from_bytes(
        &[],
        &CustomGateSerializer,
        &CustomGeneratorSerializer {
            _phantom: PhantomData::<PoseidonGoldilocksConfig>,
        },
    )
    .unwrap();

    let deposit_accumulator_leaf_targets =
        deposit_accumulator_leaf_circuit(&mut builder, &bls_data.common, &bls_data.verifier_only);

    let start = std::time::Instant::now();
    let data = builder.build::<PoseidonGoldilocksConfig>();
    let duration = start.elapsed();

    println!("Duration {:?}", duration);

    let json = json!("{
        validator: {
          pubkey: 'b781956110d24e4510a8b5500b71529f8635aa419a009d314898e8c572a4f923ba643ae94bdfdf9224509177aa8e6b73',
          withdrawalCredentials: '010000000000000000000000beefd32838d5762ff55395a7beebef6e8528c64f',
          effectiveBalance: 31000000000,
          slashed: false,
          activationEligibilityEpoch: 810,
          activationEpoch: 816,
          exitEpoch: Infinity,
          withdrawableEpoch: Infinity
        },
        validatorDeposit: {
          pubkey: 'b781956110d24e4510a8b5500b71529f8635aa419a009d314898e8c572a4f923ba643ae94bdfdf9224509177aa8e6b73',
          deposit_index: 1n,
          signature: '0xb735d0d0b03f51fcf3e5bc510b5a2cb266075322f5761a6954778714f5ab8831bc99454380d330f5c19d93436f0c4339041bfeecd2161a122c1ce8428033db8dda142768a48e582f5f9bde7d40768ac5a3b6a80492b73719f1523c5da35de275',
          deposit_message_hash_tree_root: '3c8a24bc5010fd0a28dd2448d27df4963198d84fb7c03ca3826eecd7adfedcd1'
        },
        commitmentMapperHashTreeRoot: [
          '7354616297401405606',
          '1100245580527406969',
          '10869738626706821039',
          '2491999729156780167'
        ],
        commimtnetMapperProof: [
          [ '0', '0', '0', '0' ],
          [
            '4330397376401421145',
            '14124799381142128323',
            '8742572140681234676',
            '14345658006221440202'
          ],
          [
            '13121882728673923020',
            '10197653806804742863',
            '16037207047953124082',
            '2420399206709257475'
          ],
          [
            '7052649073129349210',
            '11107139769197583972',
            '5114845353783771231',
            '7453521209854829890'
          ],
          [
            '5860469655587923524',
            '10142584705005652295',
            '1620588827255328039',
            '17663938664361140288'
          ],
          [
            '16854358529591173550',
            '9704301947898025017',
            '13222045073939169687',
            '14989445859181028978'
          ],
          [
            '2675805695450374474',
            '6493392849121218307',
            '15972287940310989584',
            '5284431416427098307'
          ],
          [
            '16823738737355150819',
            '4366876208047374841',
            '1642083707956929713',
            '13216064879834397173'
          ],
          [
            '18334109492892739862',
            '10192437552951753306',
            '15211985613247588647',
            '3157981091968158131'
          ],
          [
            '4369129498500264270',
            '10758747855946482846',
            '3238306058428322199',
            '18226589090145367109'
          ],
          [
            '14769473886748754115',
            '10513963056908986963',
            '8105478726930894327',
            '14014796621245524545'
          ],
          [
            '10191288259157808067',
            '944536249556834531',
            '16268598854718968908',
            '2417244819673331317'
          ],
          [
            '17088215091100491041',
            '18086883194773274646',
            '10296247222913205474',
            '7017044080942280524'
          ],
          [
            '2985877902215057279',
            '14516746119572211305',
            '594952314256159992',
            '17038984393731825093'
          ],
          [
            '101510842507023404',
            '2267676083447667738',
            '18106248392660779137',
            '17680390044293740318'
          ],
          [
            '16662284396446084312',
            '7269926520507830029',
            '14791338760961128332',
            '7825163129638412009'
          ],
          [
            '12364052984629808614',
            '13066500727264825316',
            '6321076066274078148',
            '11393071566019822187'
          ],
          [
            '6163084833659416779',
            '2853393070793212496',
            '214169662941198197',
            '766838854721082896'
          ],
          [
            '15062514972738604859',
            '4072732498117267624',
            '11453597623878964866',
            '15196232748141971349'
          ],
          [
            '8105799423402967201',
            '10398709180756906993',
            '12579914275816041967',
            '3722472173064824114'
          ],
          [
            '4869072528223352863',
            '6275850450145071959',
            '8159689720148436485',
            '8979985763136073723'
          ],
          [
            '8512358054591706621',
            '12918418052549764713',
            '3564884046313350424',
            '18039231110525565261'
          ],
          [
            '10074982884687544941',
            '4177217016749721471',
            '4797356481048217516',
            '6983283665462696061'
          ],
          [
            '7025400382759865156',
            '2103688473762123306',
            '8681027323514330807',
            '13853995481224614401'
          ],
          [
            '3896366420105793420',
            '17410332186442776169',
            '7329967984378645716',
            '6310665049578686403'
          ],
          [
            '6574146240104132812',
            '2239043898123515337',
            '13809601679688051486',
            '16196448971140258304'
          ],
          [
            '7429917014148897946',
            '13764740161233226515',
            '14310941960777962392',
            '10321132974520710857'
          ],
          [
            '16852763145767657080',
            '5650551567722662817',
            '4688637260797538488',
            '504212361217900660'
          ],
          [
            '17594730245457333136',
            '13719209718183388763',
            '11444947689050098668',
            '628489339233491445'
          ],
          [
            '7731246070744876899',
            '3033565575746121792',
            '14735263366152051322',
            '16212144996433476818'
          ],
          [
            '9947841139978160787',
            '692236217135079542',
            '16309341595179079658',
            '9294006745033445642'
          ],
          [
            '8603459983426387388',
            '1706773463182378335',
            '10020230853197995171',
            '2362856042482390481'
          ],
          [
            '16463394126558395459',
            '12818610997234032270',
            '2968763245313636978',
            '15445927884703223427'
          ],
          [
            '16924929798993045119',
            '9228476078763095559',
            '3639599968030750173',
            '9842693474971302918'
          ],
          [
            '2488667422532942441',
            '619530082608543022',
            '3698308124541679027',
            '1337151890861372088'
          ],
          [
            '10420632113085830027',
            '2043024317550638523',
            '9353702824282721936',
            '13923517817060358740'
          ],
          [
            '2864602688424687291',
            '3849603923476837883',
            '15617889861797529219',
            '12429234418051645329'
          ],
          [
            '2558543962574772915',
            '9272315342420626056',
            '4474448392614911585',
            '1483027055753170828'
          ],
          [
            '15131845414406822716',
            '5979581984005702075',
            '6999690762874000865',
            '9727258862093954055'
          ],
          [
            '16947881275436717432',
            '7978417559450660789',
            '5545004785373663100',
            '8368806924824039910'
          ]
        ],
        validatorIndex: 0,
        balance_tree_root: '69ebbb184788ed2675e45d4adaad12391a73d76cdd4afb05a9501eaa492c8668',
        balance_leaf: '5abde43f07000000fca5cb730b000000f67bd56f0b00000054d1dd720b000000',
        balance_proof: [
          '03051e700b00000071cf486e0b000000c5bc80830b000000f9f5aa710b000000',
          'a384b74ef15c29731b95f9c9336a84acdf2d33c4a37df85ea7f6b9ea21ee3dca',
          '91ef8edc5b3d19f4add94e5dc8510934b5089a5c325d2129d7ebbf350732b3b2',
          'c78009fdf07fc56a11f122370658a353aaa542ed63e44c4bc15ff4cd105ab33c',
          '536d98837f2dd165a55d5eeae91485954472d56f246df256bf3cae19352a123c',
          '9efde052aa15429fae05bad4d0b1d7c64da64d03d7a1854a588c2cb8430c0d30',
          'd88ddfeed400a8755596b21942c1497e114c302e6118290f91e6772976041fa1',
          '87eb0ddba57e35f6d286673802a4af5975e22506c7cf4c64bb6be5ee11527f2c',
          '26846476fd5fc54a5d43385167c95144f2643f533cc85bb9d16b782f8d7db193',
          '506d86582d252405b840018792cad2bf1259f1ef5aa5f887e13cb2f0094f51e1',
          'ffff0ad7e659772f9534c195c815efc4014ef1e1daed4404c06385d11192e92b',
          '6cf04127db05441cd833107a52be852868890e4317e6a02ab47683aa75964220',
          'b7d05f875f140027ef5118a2247bbb84ce8f2f0f1123623085daf7960c329f5f',
          'df6af5f5bbdb6be9ef8aa618e4bf8073960867171e29676f8b284dea6a08a85e',
          'b58d900f5e182e3c50ef74969ea16c7726c549757cc23523c369587da7293784',
          'd49a7502ffcfb0340b1d7885688500ca308161a7f96b62df9d083b71fcc8f2bb',
          '8fe6b1689256c0d385f42f5bbe2027a22c1996e110ba97c171d3e5948de92beb',
          '8d0d63c39ebade8509e0ae3c9c3876fb5fa112be18f905ecacfecb92057603ab',
          '95eec8b2e541cad4e91de38385f2e046619f54496c2382cb6cacd5b98c26f5a4',
          'f893e908917775b62bff23294dbbe3a1cd8e6cc1c35b4801887b646a6f81f17f',
          'cddba7b592e3133393c16194fac7431abf2f5485ed711db282183c819e08ebaa',
          '8a8d7fe3af8caa085a7639a832001457dfb9128a8061142ad0335629ff23ff9c'
        ],
        blsSignatureProofKey: 'bls12_381_b781956110d24e4510a8b5500b71529f8635aa419a009d314898e8c572a4f923ba643ae94bdfdf9224509177aa8e6b73_1',
        currentEpoch: 157708n,
        isDummy: false,
        eth1DepositIndex: 403
      }");

    let mut pw = PartialWitness::<GoldilocksField>::new();

    let validator = json.get("validator");

    pw.set_target_arr(&deposit_accumulator_leaf_targets.validator.pubkey, )
}
