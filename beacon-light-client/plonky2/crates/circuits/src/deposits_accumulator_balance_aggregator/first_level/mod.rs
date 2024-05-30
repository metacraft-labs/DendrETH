use std::str::FromStr;

use crate::{
    common_targets::{PubkeyTarget, Sha256MerkleBranchTarget, SignatureTarget},
    deposits_accumulator_balance_aggregator::common_targets::{
        AccumulatedDataTargets, RangeObjectTarget, ValidatorStatsTargets,
    },
    serializers::{
        biguint_to_str, parse_biguint, serde_bool_array_to_hex_string,
        serde_bool_array_to_hex_string_nested,
    },
    utils::circuit::{
        get_balance_from_leaf,
        hashing::{
            merkle::{
                poseidon::{hash_validator_poseidon, validate_merkle_proof_poseidon},
                sha256::{hash_tree_root_sha256, validate_merkle_proof_sha256},
            },
            sha256::sha256_pair,
        },
        select_biguint,
    },
    withdrawal_credentials_balance_aggregator::first_level::is_active_validator::get_validator_status,
};
use circuit::Circuit;
use circuit_derive::{CircuitTarget, SerdeCircuitTarget};
use num::BigUint;
use plonky2::{
    field::{extension::Extendable, goldilocks_field::GoldilocksField},
    hash::hash_types::{HashOutTarget, RichField},
    iop::target::BoolTarget,
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData, CommonCircuitData, VerifierOnlyCircuitData},
        config::{AlgebraicHasher, GenericConfig, PoseidonGoldilocksConfig},
        proof::ProofWithPublicInputsTarget,
    },
};
use plonky2_crypto::biguint::{BigUintTarget, CircuitBuilderBiguint};

use crate::common_targets::{
    DepositTargets, PoseidonMerkleBranchTarget, Sha256Target, ValidatorTarget,
};

use super::common_targets::NodeTargets;

// use super::common_targets::NodeTargets;

pub struct DepositAccumulatorBalanceAggregatorFirstLevel {}

#[derive(CircuitTarget, SerdeCircuitTarget)]
#[serde(rename_all = "camelCase")]
pub struct DepositAccumulatorBalanceAggregatorFirstLevelTargets {
    #[target(in)]
    pub validator: ValidatorTarget,
    #[target(in)]
    pub commitment_mapper_root: HashOutTarget,
    #[target(in)]
    pub commitment_mapper_proof: PoseidonMerkleBranchTarget<40>,
    #[target(in)]
    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub validator_gindex: BigUintTarget,

    #[target(in)]
    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub eth1_deposit_index: BigUintTarget,

    #[target(in)]
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub genesis_fork_version: [BoolTarget; 32],

    #[target(in)]
    pub validator_deposit: DepositTargets,
    #[target(in)]
    pub deposit_commitment_mapper_root: HashOutTarget,
    #[target(in)]
    pub validator_deposit_proof: PoseidonMerkleBranchTarget<32>,
    #[target(in)]
    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub validator_deposit_gindex: BigUintTarget,

    #[target(in)]
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub balance_tree_root: Sha256Target,
    #[target(in)]
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub balance_leaf: Sha256Target,
    #[target(in)]
    #[serde(with = "serde_bool_array_to_hex_string_nested")]
    pub balance_proof: Sha256MerkleBranchTarget<22>,
    #[target(in)]
    pub is_dummy: BoolTarget,

    #[target(in)]
    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub current_epoch: BigUintTarget,

    pub bls_signature_proof: ProofWithPublicInputsTarget<2>,

    #[target(out)]
    pub node: NodeTargets,
}

impl Circuit for DepositAccumulatorBalanceAggregatorFirstLevel {
    type F = GoldilocksField;
    const D: usize = 2;
    type C = PoseidonGoldilocksConfig;

    const CIRCUIT_CONFIG: CircuitConfig = CircuitConfig::standard_recursion_config();

    type Target = DepositAccumulatorBalanceAggregatorFirstLevelTargets;

    type Params = CircuitData<Self::F, Self::C, { Self::D }>;

    fn define(
        builder: &mut CircuitBuilder<Self::F, { Self::D }>,
        bls_circuit_data: &Self::Params,
    ) -> Self::Target {
        let input = Self::read_circuit_input_target(builder);

        // let deposit_hash_tree_root =
        //     hash_tree_root_deposit_poseidon(builder, &input.validator_deposit);

        let is_real = builder.not(input.is_dummy);

        // let is_valid = validate_merkle_proof_poseidon(
        //     builder,
        //     &deposit_hash_tree_root,
        //     &input.deposit_commitment_mapper_root,
        //     &input.validator_deposit_proof,
        //     &input.validator_deposit_gindex,
        // );

        // builder.connect(is_valid.target, is_real.target);

        let domain = compute_domain(builder, &input.genesis_fork_version);

        let message = sha256_pair(
            builder,
            &input.validator_deposit.deposit_message_root,
            &domain,
        );

        let bls_signature_proof = builder.add_virtual_proof_with_pis(&bls_circuit_data.common);

        verify_bls_signature(
            builder,
            &input.validator_deposit.pubkey,
            &input.validator_deposit.signature,
            &message,
            &bls_signature_proof,
            &bls_circuit_data.common,
            &bls_circuit_data.verifier_only,
        );

        let deposit_is_processed = builder.cmp_biguint(
            &input.validator_deposit.deposit_index,
            &input.eth1_deposit_index,
        );

        let signature_is_valid =
            BoolTarget::new_unsafe(*bls_signature_proof.public_inputs.last().unwrap());

        let validator_is_definitely_on_chain =
            builder.and(deposit_is_processed, signature_is_valid);

        let should_check_merkle_proof = builder.and(validator_is_definitely_on_chain, is_real);

        let validator_hash_tree_root = hash_validator_poseidon(builder, &input.validator);

        let validator_is_in_commitment_mapper = validate_merkle_proof_poseidon(
            builder,
            &validator_hash_tree_root,
            &input.commitment_mapper_root,
            &input.commitment_mapper_proof,
            &input.validator_gindex,
        );

        builder.connect(
            should_check_merkle_proof.target,
            validator_is_in_commitment_mapper.target,
        );

        connect_pubkeys_are_same(
            builder,
            &input.validator.pubkey,
            &input.validator_deposit.pubkey,
        );

        assert_pubkey_is_max_if_dummy(builder, &input.validator.pubkey, is_real);

        let four = builder.constant_biguint(&BigUint::from_str("4").unwrap());
        let balance_gindex = builder.div_biguint(&input.validator_gindex, &four);

        let is_valid = validate_merkle_proof_sha256(
            builder,
            &input.balance_leaf,
            &input.balance_tree_root,
            &input.balance_proof,
            &balance_gindex,
        );

        builder.connect(should_check_merkle_proof.target, is_valid.target);

        let balance_inner_index = builder.rem_biguint(&input.validator_gindex, &four);
        let balance = get_balance_from_leaf(builder, &input.balance_leaf, balance_inner_index);
        let (is_non_activated_validator, is_valid_validator, is_exited_validator) =
            get_validator_status(
                builder,
                &input.validator.activation_epoch,
                &input.current_epoch,
                &input.validator.exit_epoch,
            );

        let will_be_counted = builder.and(should_check_merkle_proof, is_valid_validator);

        let zero = builder.zero_biguint();
        let balance = select_biguint(builder, will_be_counted, &balance, &zero);

        let active_count = will_be_counted.target;

        let non_activated_count = builder
            .and(should_check_merkle_proof, is_non_activated_validator)
            .target;

        let exited_count = builder
            .and(should_check_merkle_proof, is_exited_validator)
            .target;

        let slashed_count = builder
            .and(should_check_merkle_proof, input.validator.slashed)
            .target;

        let leftmost = RangeObjectTarget {
            pubkey: input.validator.pubkey,
            deposit_index: input.validator_deposit.deposit_index.clone(),
            is_counted: should_check_merkle_proof,
            is_dummy: input.is_dummy,
        };

        let rightmost = RangeObjectTarget {
            pubkey: input.validator.pubkey,
            deposit_index: input.validator_deposit.deposit_index.clone(),
            is_counted: should_check_merkle_proof,
            is_dummy: input.is_dummy,
        };

        let one = builder.one();
        let accumulated = AccumulatedDataTargets {
            balance_sum: balance,
            deposits_count: one,
            validator_stats: ValidatorStatsTargets {
                non_activated_validators_count: non_activated_count,
                active_validators_count: active_count,
                exited_validators_count: exited_count,
                slashed_validators_count: slashed_count,
            },
        };

        let node = NodeTargets {
            leftmost,
            rightmost,
            accumulated,
            current_epoch: input.current_epoch.clone(),
            eth1_deposit_index: input.eth1_deposit_index.clone(),
            commitment_mapper_root: input.commitment_mapper_root,
            balances_root: input.balance_tree_root,
            deposits_mapper_root: input.deposit_commitment_mapper_root,
            genesis_fork_version: input.genesis_fork_version,
        };

        Self::Target {
            validator: input.validator,
            commitment_mapper_root: input.commitment_mapper_root,
            commitment_mapper_proof: input.commitment_mapper_proof,
            validator_gindex: input.validator_gindex,
            eth1_deposit_index: input.eth1_deposit_index,
            genesis_fork_version: input.genesis_fork_version,
            validator_deposit: input.validator_deposit,
            deposit_commitment_mapper_root: input.deposit_commitment_mapper_root,
            validator_deposit_proof: input.validator_deposit_proof,
            validator_deposit_gindex: input.validator_deposit_gindex,
            balance_tree_root: input.balance_tree_root,
            balance_leaf: input.balance_leaf,
            balance_proof: input.balance_proof,
            is_dummy: input.is_dummy,
            current_epoch: input.current_epoch,
            bls_signature_proof,
            node,
        }
    }
}

fn assert_pubkey_is_max_if_dummy<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    pubkey: &PubkeyTarget,
    is_real: BoolTarget,
) {
    let one = builder.one();

    for i in 0..384 {
        let is_one = builder.is_equal(pubkey[i].target, one);
        let should_be_true = builder.or(is_real, is_one);
        builder.connect(one, should_be_true.target);
    }
}

fn connect_pubkeys_are_same<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    pubkey_1: &PubkeyTarget,
    pubkey_2: &PubkeyTarget,
) {
    for i in 0..384 {
        builder.connect(pubkey_1[i].target, pubkey_2[i].target);
    }
}

pub fn compute_domain<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    genesis_fork_version: &[BoolTarget; 32],
) -> [BoolTarget; 256] {
    let zero_bits_224 = [BoolTarget::new_unsafe(builder.zero()); 224];
    let genesis_fork_version_merkelized =
        [genesis_fork_version.as_slice(), zero_bits_224.as_slice()]
            .concat()
            .try_into()
            .unwrap();

    let genesis_validator_root = [BoolTarget::new_unsafe(builder.zero()); 256];

    let fork_data_root = hash_tree_root_sha256(
        builder,
        &[genesis_fork_version_merkelized, genesis_validator_root],
    );

    let _false = builder._false();
    let _true = builder._true();

    let domain_deposit = [
        _false, _false, _false, _false, _false, _false, _true, _true, _false, _false, _false,
        _false, _false, _false, _false, _false, _false, _false, _false, _false, _false, _false,
        _false, _false, _false, _false, _false, _false, _false, _false, _false, _false,
    ];

    let domain = [domain_deposit.as_slice(), &fork_data_root[..224]]
        .concat()
        .try_into()
        .unwrap();

    domain
}

fn verify_bls_signature<F: RichField + Extendable<D>, C: GenericConfig<D, F = F>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    pubkey: &PubkeyTarget,
    signature: &SignatureTarget,
    message: &[BoolTarget; 256],
    bls_signature_proof: &ProofWithPublicInputsTarget<D>,
    bls_common_data: &CommonCircuitData<F, D>,
    bls_verifier_data: &VerifierOnlyCircuitData<C, D>,
) where
    <C as GenericConfig<D>>::Hasher: AlgebraicHasher<F>,
{
    let bls_verifier_circuit_targets = builder.constant_verifier_data(bls_verifier_data);

    builder.verify_proof::<C>(
        &bls_signature_proof,
        &bls_verifier_circuit_targets,
        bls_common_data,
    );

    for i in (0..384).step_by(8) {
        let byte = builder.le_sum(pubkey[i..(i + 8)].iter().rev().copied());
        builder.connect(byte, bls_signature_proof.public_inputs[i / 8])
    }

    for i in (0..768).step_by(8) {
        let byte = builder.le_sum(signature[i..(i + 8)].iter().rev().copied());
        builder.connect(byte, bls_signature_proof.public_inputs[(i + 384) / 8]);
    }

    for i in (0..256).step_by(8) {
        let byte = builder.le_sum(message[i..(i + 8)].iter().rev().copied());
        builder.connect(byte, bls_signature_proof.public_inputs[(i + 384 + 768) / 8]);
    }
}

#[cfg(test)]
mod test {
    use std::{fs, marker::PhantomData, time::Instant};

    use circuit::{Circuit, CircuitInput, SetWitness};
    use plonky2::{
        field::goldilocks_field::GoldilocksField,
        iop::witness::{PartialWitness, WitnessWrite},
        plonk::{
            circuit_data::CircuitData, config::PoseidonGoldilocksConfig,
            proof::ProofWithPublicInputs,
        },
    };
    use plonky2_circuit_serializer::serializer::{CustomGateSerializer, CustomGeneratorSerializer};

    use super::DepositAccumulatorBalanceAggregatorFirstLevel;

    #[test]
    pub fn test_deposit_accumulator_leaf_circuit() {
        let bls_circuit_bytes =
            fs::read("../circuit_executables/serialized_circuits/bls12_381.plonky2_circuit")
                .unwrap();

        let bls_data = CircuitData::<GoldilocksField, PoseidonGoldilocksConfig, 2>::from_bytes(
            &bls_circuit_bytes,
            &CustomGateSerializer,
            &CustomGeneratorSerializer {
                _phantom: PhantomData::<PoseidonGoldilocksConfig>,
            },
        )
        .unwrap();

        let s = Instant::now();
        println!("Stared building circuit");
        let (targets, circuit) = DepositAccumulatorBalanceAggregatorFirstLevel::build(&bls_data);
        println!("Circuit built in {:?}", s.elapsed());

        let json_input = serde_json::from_str::<
            CircuitInput<DepositAccumulatorBalanceAggregatorFirstLevel>,
        >(
            r#"{
                "validator": {
                  "pubkey": "b781956110d24e4510a8b5500b71529f8635aa419a009d314898e8c572a4f923ba643ae94bdfdf9224509177aa8e6b73",
                  "withdrawalCredentials": "010000000000000000000000beefd32838d5762ff55395a7beebef6e8528c64f",
                  "effectiveBalance": "31000000000",
                  "slashed": false,
                  "activationEligibilityEpoch": "810",
                  "activationEpoch": "816",
                  "exitEpoch": "18446744073709551615",
                  "withdrawableEpoch": "18446744073709551615"
                },
                "validatorDeposit": {
                  "pubkey": "b781956110d24e4510a8b5500b71529f8635aa419a009d314898e8c572a4f923ba643ae94bdfdf9224509177aa8e6b73",
                  "depositIndex": "1486868",
                  "signature": "b735d0d0b03f51fcf3e5bc510b5a2cb266075322f5761a6954778714f5ab8831bc99454380d330f5c19d93436f0c4339041bfeecd2161a122c1ce8428033db8dda142768a48e582f5f9bde7d40768ac5a3b6a80492b73719f1523c5da35de275",
                  "depositMessageRoot": "3c8a24bc5010fd0a28dd2448d27df4963198d84fb7c03ca3826eecd7adfedcd1"
                },
                "commitmentMapperRoot": [
                  9598574192830158314,
                  15798717172533122073,
                  2738900304525190430,
                  18373424356305701427
                ],
                "depositCommitmentMapperRoot": [
                  10029351254276310541,
                  17489004309530712790,
                  14045656543692830747,
                  14925290861725349124
                ],
                "validatorDepositProof": [
                  [
                    0,
                    0,
                    0,
                    0
                  ],
                  [
                    4330397376401421145,
                    14124799381142128323,
                    8742572140681234676,
                    14345658006221440202
                  ],
                  [
                    13121882728673923020,
                    10197653806804742863,
                    16037207047953124082,
                    2420399206709257475
                  ],
                  [
                    7052649073129349210,
                    11107139769197583972,
                    5114845353783771231,
                    7453521209854829890
                  ],
                  [
                    5860469655587923524,
                    10142584705005652295,
                    1620588827255328039,
                    17663938664361140288
                  ],
                  [
                    16854358529591173550,
                    9704301947898025017,
                    13222045073939169687,
                    14989445859181028978
                  ],
                  [
                    2675805695450374474,
                    6493392849121218307,
                    15972287940310989584,
                    5284431416427098307
                  ],
                  [
                    16823738737355150819,
                    4366876208047374841,
                    1642083707956929713,
                    13216064879834397173
                  ],
                  [
                    18334109492892739862,
                    10192437552951753306,
                    15211985613247588647,
                    3157981091968158131
                  ],
                  [
                    4369129498500264270,
                    10758747855946482846,
                    3238306058428322199,
                    18226589090145367109
                  ],
                  [
                    14769473886748754115,
                    10513963056908986963,
                    8105478726930894327,
                    14014796621245524545
                  ],
                  [
                    10191288259157808067,
                    944536249556834531,
                    16268598854718968908,
                    2417244819673331317
                  ],
                  [
                    17088215091100491041,
                    18086883194773274646,
                    10296247222913205474,
                    7017044080942280524
                  ],
                  [
                    2985877902215057279,
                    14516746119572211305,
                    594952314256159992,
                    17038984393731825093
                  ],
                  [
                    101510842507023404,
                    2267676083447667738,
                    18106248392660779137,
                    17680390044293740318
                  ],
                  [
                    16662284396446084312,
                    7269926520507830029,
                    14791338760961128332,
                    7825163129638412009
                  ],
                  [
                    12364052984629808614,
                    13066500727264825316,
                    6321076066274078148,
                    11393071566019822187
                  ],
                  [
                    6163084833659416779,
                    2853393070793212496,
                    214169662941198197,
                    766838854721082896
                  ],
                  [
                    15062514972738604859,
                    4072732498117267624,
                    11453597623878964866,
                    15196232748141971349
                  ],
                  [
                    8105799423402967201,
                    10398709180756906993,
                    12579914275816041967,
                    3722472173064824114
                  ],
                  [
                    4869072528223352863,
                    6275850450145071959,
                    8159689720148436485,
                    8979985763136073723
                  ],
                  [
                    8512358054591706621,
                    12918418052549764713,
                    3564884046313350424,
                    18039231110525565261
                  ],
                  [
                    10074982884687544941,
                    4177217016749721471,
                    4797356481048217516,
                    6983283665462696061
                  ],
                  [
                    7025400382759865156,
                    2103688473762123306,
                    8681027323514330807,
                    13853995481224614401
                  ],
                  [
                    3896366420105793420,
                    17410332186442776169,
                    7329967984378645716,
                    6310665049578686403
                  ],
                  [
                    6574146240104132812,
                    2239043898123515337,
                    13809601679688051486,
                    16196448971140258304
                  ],
                  [
                    7429917014148897946,
                    13764740161233226515,
                    14310941960777962392,
                    10321132974520710857
                  ],
                  [
                    16852763145767657080,
                    5650551567722662817,
                    4688637260797538488,
                    504212361217900660
                  ],
                  [
                    17594730245457333136,
                    13719209718183388763,
                    11444947689050098668,
                    628489339233491445
                  ],
                  [
                    7731246070744876899,
                    3033565575746121792,
                    14735263366152051322,
                    16212144996433476818
                  ],
                  [
                    9947841139978160787,
                    692236217135079542,
                    16309341595179079658,
                    9294006745033445642
                  ],
                  [
                    8603459983426387388,
                    1706773463182378335,
                    10020230853197995171,
                    2362856042482390481
                  ]
                ],
                "validatorDepositGindex": "1",
                "commitmentMapperProof": [
                  [
                    14253833605643055169,
                    573597012073253524,
                    10786694560502154666,
                    2029558398106597126
                  ],
                  [
                    13984414887454990918,
                    10294358825814302131,
                    2256206737430167672,
                    5245051478213075588
                  ],
                  [
                    4261432127699126961,
                    6622988869022885260,
                    14700606944341294125,
                    11433338254825916872
                  ],
                  [
                    3519327814879640916,
                    484815144706572751,
                    8372415782774993735,
                    12367562363689062942
                  ],
                  [
                    6576323203091444448,
                    2255221132679866028,
                    5095666707065713784,
                    10098008411061433956
                  ],
                  [
                    16854358529591173550,
                    9704301947898025017,
                    13222045073939169687,
                    14989445859181028978
                  ],
                  [
                    2675805695450374474,
                    6493392849121218307,
                    15972287940310989584,
                    5284431416427098307
                  ],
                  [
                    16823738737355150819,
                    4366876208047374841,
                    1642083707956929713,
                    13216064879834397173
                  ],
                  [
                    18334109492892739862,
                    10192437552951753306,
                    15211985613247588647,
                    3157981091968158131
                  ],
                  [
                    4369129498500264270,
                    10758747855946482846,
                    3238306058428322199,
                    18226589090145367109
                  ],
                  [
                    14769473886748754115,
                    10513963056908986963,
                    8105478726930894327,
                    14014796621245524545
                  ],
                  [
                    10191288259157808067,
                    944536249556834531,
                    16268598854718968908,
                    2417244819673331317
                  ],
                  [
                    17088215091100491041,
                    18086883194773274646,
                    10296247222913205474,
                    7017044080942280524
                  ],
                  [
                    2985877902215057279,
                    14516746119572211305,
                    594952314256159992,
                    17038984393731825093
                  ],
                  [
                    101510842507023404,
                    2267676083447667738,
                    18106248392660779137,
                    17680390044293740318
                  ],
                  [
                    16662284396446084312,
                    7269926520507830029,
                    14791338760961128332,
                    7825163129638412009
                  ],
                  [
                    12364052984629808614,
                    13066500727264825316,
                    6321076066274078148,
                    11393071566019822187
                  ],
                  [
                    6163084833659416779,
                    2853393070793212496,
                    214169662941198197,
                    766838854721082896
                  ],
                  [
                    15062514972738604859,
                    4072732498117267624,
                    11453597623878964866,
                    15196232748141971349
                  ],
                  [
                    8105799423402967201,
                    10398709180756906993,
                    12579914275816041967,
                    3722472173064824114
                  ],
                  [
                    4869072528223352863,
                    6275850450145071959,
                    8159689720148436485,
                    8979985763136073723
                  ],
                  [
                    8512358054591706621,
                    12918418052549764713,
                    3564884046313350424,
                    18039231110525565261
                  ],
                  [
                    10074982884687544941,
                    4177217016749721471,
                    4797356481048217516,
                    6983283665462696061
                  ],
                  [
                    7025400382759865156,
                    2103688473762123306,
                    8681027323514330807,
                    13853995481224614401
                  ],
                  [
                    3896366420105793420,
                    17410332186442776169,
                    7329967984378645716,
                    6310665049578686403
                  ],
                  [
                    6574146240104132812,
                    2239043898123515337,
                    13809601679688051486,
                    16196448971140258304
                  ],
                  [
                    7429917014148897946,
                    13764740161233226515,
                    14310941960777962392,
                    10321132974520710857
                  ],
                  [
                    16852763145767657080,
                    5650551567722662817,
                    4688637260797538488,
                    504212361217900660
                  ],
                  [
                    17594730245457333136,
                    13719209718183388763,
                    11444947689050098668,
                    628489339233491445
                  ],
                  [
                    7731246070744876899,
                    3033565575746121792,
                    14735263366152051322,
                    16212144996433476818
                  ],
                  [
                    9947841139978160787,
                    692236217135079542,
                    16309341595179079658,
                    9294006745033445642
                  ],
                  [
                    8603459983426387388,
                    1706773463182378335,
                    10020230853197995171,
                    2362856042482390481
                  ],
                  [
                    16463394126558395459,
                    12818610997234032270,
                    2968763245313636978,
                    15445927884703223427
                  ],
                  [
                    16924929798993045119,
                    9228476078763095559,
                    3639599968030750173,
                    9842693474971302918
                  ],
                  [
                    2488667422532942441,
                    619530082608543022,
                    3698308124541679027,
                    1337151890861372088
                  ],
                  [
                    10420632113085830027,
                    2043024317550638523,
                    9353702824282721936,
                    13923517817060358740
                  ],
                  [
                    2864602688424687291,
                    3849603923476837883,
                    15617889861797529219,
                    12429234418051645329
                  ],
                  [
                    2558543962574772915,
                    9272315342420626056,
                    4474448392614911585,
                    1483027055753170828
                  ],
                  [
                    15131845414406822716,
                    5979581984005702075,
                    6999690762874000865,
                    9727258862093954055
                  ],
                  [
                    16947881275436717432,
                    7978417559450660789,
                    5545004785373663100,
                    8368806924824039910
                  ]
                ],
                "genesisForkVersion": "90000069",
                "validatorGindex": "1099511627776",
                "balanceTreeRoot": "20fe0fb226a1c08e1830dfab419b67caea4f4d090b7b5a73e8b9c2439b60611d",
                "balanceLeaf": "b07ad63907000000045d8b6d0b000000be642c690b0000001cba346c0b000000",
                "balanceProof": [
                  "cbf1a3690b000000798608680b000000cd73407d0b00000001ad6a6b0b000000",
                  "34f735cad9ae2d061fbab0682064d1b37e8c227e0f13e07457ce12d69e97da43",
                  "efb80785674ab41400abe50d7b3b837128ac54451ae0bf433cb9e4d9cbfc6c4c",
                  "c78009fdf07fc56a11f122370658a353aaa542ed63e44c4bc15ff4cd105ab33c",
                  "536d98837f2dd165a55d5eeae91485954472d56f246df256bf3cae19352a123c",
                  "9efde052aa15429fae05bad4d0b1d7c64da64d03d7a1854a588c2cb8430c0d30",
                  "d88ddfeed400a8755596b21942c1497e114c302e6118290f91e6772976041fa1",
                  "87eb0ddba57e35f6d286673802a4af5975e22506c7cf4c64bb6be5ee11527f2c",
                  "26846476fd5fc54a5d43385167c95144f2643f533cc85bb9d16b782f8d7db193",
                  "506d86582d252405b840018792cad2bf1259f1ef5aa5f887e13cb2f0094f51e1",
                  "ffff0ad7e659772f9534c195c815efc4014ef1e1daed4404c06385d11192e92b",
                  "6cf04127db05441cd833107a52be852868890e4317e6a02ab47683aa75964220",
                  "b7d05f875f140027ef5118a2247bbb84ce8f2f0f1123623085daf7960c329f5f",
                  "df6af5f5bbdb6be9ef8aa618e4bf8073960867171e29676f8b284dea6a08a85e",
                  "b58d900f5e182e3c50ef74969ea16c7726c549757cc23523c369587da7293784",
                  "d49a7502ffcfb0340b1d7885688500ca308161a7f96b62df9d083b71fcc8f2bb",
                  "8fe6b1689256c0d385f42f5bbe2027a22c1996e110ba97c171d3e5948de92beb",
                  "8d0d63c39ebade8509e0ae3c9c3876fb5fa112be18f905ecacfecb92057603ab",
                  "95eec8b2e541cad4e91de38385f2e046619f54496c2382cb6cacd5b98c26f5a4",
                  "f893e908917775b62bff23294dbbe3a1cd8e6cc1c35b4801887b646a6f81f17f",
                  "cddba7b592e3133393c16194fac7431abf2f5485ed711db282183c819e08ebaa",
                  "8a8d7fe3af8caa085a7639a832001457dfb9128a8061142ad0335629ff23ff9c"
                ],
                "blsSignatureProofKey": "bls12_381_b781956110d24e4510a8b5500b71529f8635aa419a009d314898e8c572a4f923ba643ae94bdfdf9224509177aa8e6b73_1",
                "currentEpoch": "158342",
                "isDummy": false,
                "eth1DepositIndex": "403"
              }
              "#,
        ).unwrap();

        let proof_bytes = fs::read("bls12_381_proof").unwrap();
        let bls_proof =
            ProofWithPublicInputs::<GoldilocksField, PoseidonGoldilocksConfig, 2>::from_bytes(
                proof_bytes,
                &bls_data.common,
            )
            .unwrap();

        println!("bls public inputs: {:?}", bls_proof.public_inputs);

        let mut pw = PartialWitness::<GoldilocksField>::new();
        targets.set_witness(&mut pw, &json_input);
        pw.set_proof_with_pis_target(&targets.bls_signature_proof, &bls_proof);

        let s = Instant::now();
        let proof = circuit.prove(pw).unwrap();
        println!("Proof generated in {:?}", s.elapsed());

        let result =
            DepositAccumulatorBalanceAggregatorFirstLevel::read_public_inputs(&proof.public_inputs);

        let result_str = serde_json::to_string(&result).unwrap();
        println!("Public inputs: {:?}", result_str);
    }
}
