use crate::{
    bls_verification::bls12_381_circuit::BLSVerificationCircuit,
    common_targets::Sha256MerkleBranchTarget,
    deposits_accumulator_balance_aggregator::common_targets::{
        AccumulatedDataTarget, DepositDataTarget, ValidatorStatusStatsTarget,
    },
    serializers::{
        biguint_to_str, parse_biguint, serde_bool_array_to_hex_string,
        serde_bool_array_to_hex_string_nested,
    },
    utils::circuit::{
        assert_arrays_are_equal, assert_bool_arrays_are_equal, bits_to_bytes_target,
        get_balance_from_leaf,
        hashing::{
            merkle::{
                poseidon::{hash_validator_poseidon, validate_merkle_proof_poseidon},
                sha256::{hash_tree_root_sha256, validate_merkle_proof_sha256},
            },
            poseidon::poseidon,
            sha256::sha256_pair,
        },
        select_biguint,
        validator_status::{get_validator_relevance, get_validator_status},
        verify_proof,
    },
};
use circuit::{
    circuit_builder_extensions::CircuitBuilderExtensions, Circuit, CircuitInputTarget, ToTargets,
};
use circuit_derive::{CircuitTarget, SerdeCircuitTarget};
use num::{BigUint, FromPrimitive};
use plonky2::{
    field::{extension::Extendable, goldilocks_field::GoldilocksField},
    hash::hash_types::{HashOutTarget, RichField},
    iop::target::BoolTarget,
    plonk::{
        circuit_builder::CircuitBuilder,
        circuit_data::{CircuitConfig, CircuitData},
        config::{AlgebraicHasher, GenericConfig, PoseidonGoldilocksConfig},
        proof::ProofWithPublicInputsTarget,
    },
};
use plonky2_crypto::biguint::{BigUintTarget, CircuitBuilderBiguint};

use crate::common_targets::{
    DepositTargets, PoseidonMerkleBranchTarget, Sha256Target, ValidatorTarget,
};

use super::common_targets::ValidatorDataTarget;

pub struct DepositAccumulatorBalanceAggregatorFirstLevel {}

#[derive(CircuitTarget, SerdeCircuitTarget)]
#[serde(rename_all = "camelCase")]
pub struct DepositAccumulatorBalanceAggregatorFirstLevelTargets {
    #[target(in)]
    pub validator: ValidatorTarget,

    #[target(in)]
    pub commitment_mapper_proof: PoseidonMerkleBranchTarget<40>,

    #[target(in)]
    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub validator_gindex: BigUintTarget,

    #[target(in)]
    pub deposit: DepositTargets,

    #[target(in)]
    pub validator_deposit_proof: PoseidonMerkleBranchTarget<32>,

    #[target(in)]
    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub validator_deposit_gindex: BigUintTarget,

    #[target(in)]
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub balance_leaf: Sha256Target,

    #[target(in)]
    #[serde(with = "serde_bool_array_to_hex_string_nested")]
    pub balance_proof: Sha256MerkleBranchTarget<22>,

    #[target(in)]
    pub is_dummy: BoolTarget,

    #[target(in, out)]
    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub current_epoch: BigUintTarget,

    #[target(in, out)]
    #[serde(serialize_with = "biguint_to_str", deserialize_with = "parse_biguint")]
    pub eth1_deposit_index: BigUintTarget,

    #[target(in, out)]
    pub commitment_mapper_root: HashOutTarget,

    #[target(in, out)]
    pub deposits_commitment_mapper_root: HashOutTarget,

    #[target(in, out)]
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub balances_root: Sha256Target,

    #[target(in, out)]
    #[serde(with = "serde_bool_array_to_hex_string")]
    pub genesis_fork_version: [BoolTarget; 32],

    #[target(out)]
    pub leftmost_deposit: DepositDataTarget,

    #[target(out)]
    pub rightmost_deposit: DepositDataTarget,

    #[target(out)]
    pub accumulated_data: AccumulatedDataTarget,

    pub bls_verification_proof: ProofWithPublicInputsTarget<2>,
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

        let deposit_hash_tree_root = poseidon(builder, input.deposit.to_targets());

        let deposit_is_real = builder.not(input.is_dummy);

        let is_valid = validate_merkle_proof_poseidon(
            builder,
            &deposit_hash_tree_root,
            &input.deposits_commitment_mapper_root,
            &input.validator_deposit_proof,
            &input.validator_deposit_gindex,
        );

        builder.assert_implication(deposit_is_real, is_valid);

        let (bls_verification_proof, signature_is_valid) =
            verify_bls_signature(builder, &input, &bls_circuit_data);

        let deposit_is_processed =
            builder.cmp_biguint(&input.deposit.deposit_index, &input.eth1_deposit_index);

        let validator_is_on_chain = builder.and(deposit_is_processed, signature_is_valid);
        let deposit_is_real_and_validator_is_on_chain =
            builder.and(validator_is_on_chain, deposit_is_real);

        let validator_hash_tree_root = hash_validator_poseidon(builder, &input.validator);

        let validators_proof_is_valid = validate_merkle_proof_poseidon(
            builder,
            &validator_hash_tree_root,
            &input.commitment_mapper_root,
            &input.commitment_mapper_proof,
            &input.validator_gindex,
        );

        builder.assert_implication(
            deposit_is_real_and_validator_is_on_chain,
            validators_proof_is_valid,
        );

        assert_bool_arrays_are_equal(builder, &input.validator.pubkey, &input.deposit.pubkey);

        let four = builder.constant_biguint(&BigUint::from_u64(4u64).unwrap());
        let balance_inner_index = builder.rem_biguint(&input.validator_gindex, &four);
        let balance = get_balance_from_leaf(builder, &input.balance_leaf, balance_inner_index);
        let balance_gindex = builder.div_biguint(&input.validator_gindex, &four);

        let balances_proof_is_valid = validate_merkle_proof_sha256(
            builder,
            &input.balance_leaf,
            &input.balances_root,
            &input.balance_proof,
            &balance_gindex,
        );

        builder.assert_implication(
            deposit_is_real_and_validator_is_on_chain,
            balances_proof_is_valid,
        );

        let (is_non_activated, is_active, is_exited) = get_validator_status(
            builder,
            &input.validator.activation_epoch,
            &input.current_epoch,
            &input.validator.exit_epoch,
        );

        let zero_validator_status_stats: ValidatorStatusStatsTarget = builder.zero_init();
        let mut validator_status_stats = ValidatorStatusStatsTarget {
            non_activated_count: is_non_activated.target,
            active_count: is_active.target,
            exited_count: is_exited.target,
            slashed_count: input.validator.slashed.target,
        };
        validator_status_stats = builder.select_target(
            deposit_is_real_and_validator_is_on_chain,
            &validator_status_stats,
            &zero_validator_status_stats,
        );

        let mut is_relevant = get_validator_relevance(
            builder,
            &input.validator.activation_epoch,
            &input.current_epoch,
            &input.validator.withdrawable_epoch,
        );

        is_relevant = builder.and(is_relevant, deposit_is_real_and_validator_is_on_chain);

        let zero_biguint = builder.zero_biguint();
        let validator_balance = select_biguint(builder, is_relevant, &balance, &zero_biguint);

        let deposit_data = DepositDataTarget {
            pubkey: input.deposit.pubkey,
            validator: ValidatorDataTarget {
                balance: validator_balance.clone(),
                is_non_activated,
                is_active,
                is_exited,
                is_slashed: input.validator.slashed,
            },
            deposit_index: input.deposit.deposit_index.clone(),
            is_counted: deposit_is_real_and_validator_is_on_chain,
            is_dummy: input.is_dummy,
        };

        let zero_accumulated_data: AccumulatedDataTarget = builder.zero_init();
        let mut accumulated_data = AccumulatedDataTarget {
            balance: validator_balance,
            deposits_count: builder.one(),
            validator_status_stats,
        };
        accumulated_data =
            builder.select_target(deposit_is_real, &accumulated_data, &zero_accumulated_data);

        Self::Target {
            leftmost_deposit: deposit_data.clone(),
            rightmost_deposit: deposit_data.clone(),
            accumulated_data,

            validator: input.validator,
            commitment_mapper_root: input.commitment_mapper_root,
            commitment_mapper_proof: input.commitment_mapper_proof,
            validator_gindex: input.validator_gindex,
            eth1_deposit_index: input.eth1_deposit_index,
            genesis_fork_version: input.genesis_fork_version,
            deposit: input.deposit,
            deposits_commitment_mapper_root: input.deposits_commitment_mapper_root,
            validator_deposit_proof: input.validator_deposit_proof,
            validator_deposit_gindex: input.validator_deposit_gindex,
            balances_root: input.balances_root,
            balance_leaf: input.balance_leaf,
            balance_proof: input.balance_proof,
            is_dummy: input.is_dummy,
            current_epoch: input.current_epoch,

            bls_verification_proof,
        }
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

    let genesis_validator_root = [builder._false(); 256];

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
    input: &CircuitInputTarget<DepositAccumulatorBalanceAggregatorFirstLevel>,
    bls_verification_circuit_data: &CircuitData<F, C, D>,
) -> (ProofWithPublicInputsTarget<D>, BoolTarget)
where
    <C as GenericConfig<D>>::Hasher: AlgebraicHasher<F>,
{
    let bls_verification_proof = verify_proof(builder, bls_verification_circuit_data);
    let public_inputs =
        BLSVerificationCircuit::read_public_inputs_target(&bls_verification_proof.public_inputs);

    let domain = compute_domain(builder, &input.genesis_fork_version);
    let message = sha256_pair(builder, &input.deposit.deposit_message_root, &domain);

    let pubkey_bytes = bits_to_bytes_target(builder, &input.deposit.pubkey);
    assert_arrays_are_equal(builder, &pubkey_bytes, &public_inputs.pubkey);

    let message_bytes = bits_to_bytes_target(builder, &message);
    assert_arrays_are_equal(builder, &message_bytes, &public_inputs.msg);

    let signature_bytes = bits_to_bytes_target(builder, &input.deposit.signature);
    assert_arrays_are_equal(builder, &signature_bytes, &public_inputs.sig);

    (bls_verification_proof, public_inputs.is_valid_signature)
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
    #[ignore]
    // Ignored because it requires circuit data for the bls to be generated
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
                "deposit": {
                  "pubkey": "b781956110d24e4510a8b5500b71529f8635aa419a009d314898e8c572a4f923ba643ae94bdfdf9224509177aa8e6b73",
                  "depositIndex": "1486869",
                  "signature": "b735d0d0b03f51fcf3e5bc510b5a2cb266075322f5761a6954778714f5ab8831bc99454380d330f5c19d93436f0c4339041bfeecd2161a122c1ce8428033db8dda142768a48e582f5f9bde7d40768ac5a3b6a80492b73719f1523c5da35de275",
                  "depositMessageRoot": "3c8a24bc5010fd0a28dd2448d27df4963198d84fb7c03ca3826eecd7adfedcd1"
                },
                "commitmentMapperRoot": [
                  9598574192830158314,
                  15798717172533122073,
                  2738900304525190430,
                  18373424356305701427
                ],
                "depositsCommitmentMapperRoot":[
                    9753968952574624528,
                    12247890125210169943,
                    13348648846156651925,
                    13645358033895492342
                  ],
                  "validatorDepositProof": [
                    [
                      15734155317984503951,
                      12617369191460452049,
                      17451490542598576015,
                      8240955125118605513
                    ],
                    [
                      11564163488300737602,
                      4920689194753410064,
                      8637664108631438095,
                      9576195275931447952
                    ],
                    [
                      4369539326585169185,
                      791428159025329397,
                      9378532614078084704,
                      10109217897996523973
                    ],
                    [
                      15738095886659795751,
                      18135393468951797927,
                      15398673764901459885,
                      3303313015276053344
                    ],
                    [
                      4204311716293759975,
                      8218063559325857058,
                      4949948764679660978,
                      11676634994537300002
                    ],
                    [
                      4912204992629347430,
                      12717791945876082833,
                      6272100436917601979,
                      13912266007209957763
                    ],
                    [
                      176372821385248995,
                      1010183381425006539,
                      8243138397801958772,
                      12480282812008613783
                    ],
                    [
                      7248091097474755797,
                      8599100350219027037,
                      9415308642717480497,
                      5617112133311269662
                    ],
                    [
                      3522885800373479446,
                      10398092532179673098,
                      4037378614826158733,
                      8050638041088056135
                    ],
                    [
                      3476065630387752496,
                      6944058388002217919,
                      11255423548319917904,
                      1333261830837694009
                    ],
                    [
                      4967950925442832982,
                      2159929167594922918,
                      10386207707554542425,
                      10569954417099462850
                    ],
                    [
                      4223252597371858785,
                      8333548551428027731,
                      16277194138022314928,
                      5145192004039874028
                    ],
                    [
                      7329516157679608292,
                      16532576403444044404,
                      802118230491679842,
                      54121779468832286
                    ],
                    [
                      18137512065462758782,
                      12418506414595384926,
                      3385125982237646517,
                      18230955966145681284
                    ],
                    [
                      12217415923762135856,
                      5954040275915552926,
                      9380040669964194936,
                      4827844643129659033
                    ],
                    [
                      15440112015924210785,
                      6198506122217966892,
                      4058446451084276470,
                      8655907674508206569
                    ],
                    [
                      14675366402291988374,
                      2726820262598601780,
                      4482337233401578188,
                      16911869197470362757
                    ],
                    [
                      2859210824287832911,
                      2753531472406893648,
                      16260024168798100821,
                      2786837249838086261
                    ],
                    [
                      2810155980779138322,
                      4808305343627001306,
                      4992349471806615819,
                      4496808937040630424
                    ],
                    [
                      16168560411067683100,
                      6485424322116677632,
                      18401950568970748390,
                      9757881912781803868
                    ],
                    [
                      10412558578727136727,
                      18117791138906057873,
                      4574518950740553277,
                      4182229051155029484
                    ],
                    [
                      10424333753376082055,
                      8577818969223593110,
                      14599103333006561406,
                      17629072702511710906
                    ],
                    [
                      8468763291585116640,
                      8302196073336935987,
                      11158520244924912198,
                      4316290239501839518
                    ],
                    [
                      5628527647717607981,
                      4752442041992582522,
                      12286995700578714191,
                      2580881100144395493
                    ],
                    [
                      10701729761890665981,
                      9739543544869690929,
                      4738813153451540396,
                      17999482841150581739
                    ],
                    [
                      14613226658479001288,
                      3945423200381639007,
                      3626155189824512220,
                      10444086080078135569
                    ],
                    [
                      12795702992636685490,
                      5743137585794669939,
                      6620892250023242804,
                      1267260047223009438
                    ],
                    [
                      7453245113316235834,
                      132586452656262933,
                      6100922803048079075,
                      8404310923434047542
                    ],
                    [
                      9749779380636462963,
                      17319500257853287121,
                      16995163567599618338,
                      2098370039962703845
                    ],
                    [
                      13855521171206348442,
                      12200752896343011896,
                      14865803336422023209,
                      11310319634838352787
                    ],
                    [
                      15744922126217803470,
                      12969051516142104954,
                      228771806826154965,
                      16055628850963142894
                    ],
                    [
                      16585314222869428102,
                      12939627183598825729,
                      2914331685855892577,
                      17821833154649415193
                    ]
                  ],
                "validatorDepositGindex": "4294967296",
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
                "balancesRoot": "20fe0fb226a1c08e1830dfab419b67caea4f4d090b7b5a73e8b9c2439b60611d",
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

        // println!("bls public inputs: {:?}", bls_proof.public_inputs);

        let mut pw = PartialWitness::<GoldilocksField>::new();
        targets.set_witness(&mut pw, &json_input);
        pw.set_proof_with_pis_target(&targets.bls_verification_proof, &bls_proof);

        let s = Instant::now();
        let _proof = circuit.prove(pw).unwrap();
        println!("Proof generated in {:?}", s.elapsed());
    }
}
