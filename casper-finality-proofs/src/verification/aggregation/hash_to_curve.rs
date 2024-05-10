use std::str::FromStr;

use num_bigint::{BigUint, ToBigUint};
use plonky2::{
    field::extension::Extendable,
    hash::hash_types::RichField,
    iop::{
        generator::{GeneratedValues, SimpleGenerator},
        target::{BoolTarget, Target},
        witness::{PartitionWitness, WitnessWrite},
    },
    plonk::circuit_data::CommonCircuitData,
    util::serialization::{Buffer, IoResult, Read, Write},
};
use plonky2x::{
    backend::circuit::PlonkParameters,
    frontend::{
        builder::CircuitBuilder,
        uint::num::{
            biguint::{
                BigUintTarget, CircuitBuilderBiguint, GeneratedValuesBigUint, WitnessBigUint,
            },
            u32::gadgets::arithmetic_u32::U32Target,
        },
        vars::Variable,
    },
};

use crate::verification::{
    curves::g2::{
        g2_add, g2_add_without_generator, g2_double, g2_negate, g2_scalar_mul, PointG2Target,
    },
    fields::{
        fp::{mul_fp, N},
        fp2::{
            add_fp2, div_fp2, frobenius_map, is_zero, mul_fp2, negate_fp2, range_check_fp2,
            sgn0_fp2, Fp2Target,
        },
    },
    utils::native_bls::{modulus, Fp, Fp2, Pow},
};

use super::hash_to_field::hash_to_field;

pub const BLS_X: u64 = 0xd201000000010000;

pub const ISOGENY_COEFFICIENTS_G2: [[[&str; 2]; 4]; 4] = [
    [
        [
            "3557697382419259905260257622876359250272784728834673675850718343221361467102966990615722337003569479144794908942033",
            "0",
        ],
        [
            "2668273036814444928945193217157269437704588546626005256888038757416021100327225242961791752752677109358596181706526",
            "1334136518407222464472596608578634718852294273313002628444019378708010550163612621480895876376338554679298090853261",
        ],
        [
            "0",
            "2668273036814444928945193217157269437704588546626005256888038757416021100327225242961791752752677109358596181706522",
        ],
        [
            "889424345604814976315064405719089812568196182208668418962679585805340366775741747653930584250892369786198727235542",
            "889424345604814976315064405719089812568196182208668418962679585805340366775741747653930584250892369786198727235542",
        ],
    ],
    [
        [
            "0",
            "0",
        ],
        [
            "1",
            "0",
        ],
        [
            "12",
            "4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559775",
        ],
        [
            "0",
            "4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559715",
        ],
    ],
    [
        [
            "2816510427748580758331037284777117739799287910327449993381818688383577828123182200904113516794492504322962636245776",
            "0",
        ],
        [
            "2668273036814444928945193217157269437704588546626005256888038757416021100327225242961791752752677109358596181706524",
            "1334136518407222464472596608578634718852294273313002628444019378708010550163612621480895876376338554679298090853263",
        ],
        [
            "0",
            "889424345604814976315064405719089812568196182208668418962679585805340366775741747653930584250892369786198727235518",
        ],
        [
            "3261222600550988246488569487636662646083386001431784202863158481286248011511053074731078808919938689216061999863558",
            "3261222600550988246488569487636662646083386001431784202863158481286248011511053074731078808919938689216061999863558",
        ],
    ],
    [
        [
            "1",
            "0",
        ],
        [
            "18",
            "4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559769",
        ],
        [
            "0",
            "4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559571",
        ],
        [
            "4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559355",
            "4002409555221667393417789825735904156556882819939007885332058136124031650490837864442687629129015664037894272559355",
        ],
    ],
];

pub fn map_to_curve_simple_swu_9mod16<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    t: &Fp2Target,
) -> PointG2Target {
    let zero = builder.api.zero();

    let iso_3_a = [
        builder.api.constant_biguint(&0.to_biguint().unwrap()),
        builder.api.constant_biguint(&240.to_biguint().unwrap()),
    ];
    let iso_3_b = [
        builder.api.constant_biguint(&1012.to_biguint().unwrap()),
        builder.api.constant_biguint(&1012.to_biguint().unwrap()),
    ];
    let iso_3_z = [
        builder.api.constant_biguint(&(modulus() - 2u32)),
        builder.api.constant_biguint(&(modulus() - 1u32)),
    ];
    let one = [
        builder.api.constant_biguint(&1.to_biguint().unwrap()),
        builder.api.constant_biguint(&0.to_biguint().unwrap()),
    ];

    let t2 = mul_fp2(builder, &t, &t);
    let iso_3_z_t2 = mul_fp2(builder, &iso_3_z, &t2);
    let iso_3_z_t2_2 = mul_fp2(builder, &iso_3_z_t2, &iso_3_z_t2);
    let ztzt = add_fp2(builder, &iso_3_z_t2, &iso_3_z_t2_2);
    let iso_3_a_ztzt = mul_fp2(builder, &iso_3_a, &ztzt);
    let denominator_tmp = negate_fp2(builder, &iso_3_a_ztzt);
    let ztzt_1 = add_fp2(builder, &ztzt, &one);
    let numerator = mul_fp2(builder, &iso_3_b, &ztzt_1);

    let cmp = is_zero(builder, &denominator_tmp);
    let iso_3_z_iso_3_a = [
        builder.api.constant_biguint(&240.to_biguint().unwrap()),
        builder.api.constant_biguint(&(modulus() - 480u32)),
    ];
    let denominator = [
        BigUintTarget {
            limbs: (0..N)
                .into_iter()
                .map(|i| {
                    U32Target::from_target_unsafe(if i < iso_3_z_iso_3_a[0].num_limbs() {
                        builder.api.select(
                            cmp.into(),
                            iso_3_z_iso_3_a[0].limbs[i].target,
                            denominator_tmp[0].limbs[i].target,
                        )
                    } else {
                        builder
                            .api
                            .select(cmp.into(), zero, denominator_tmp[0].limbs[i].target)
                    })
                })
                .collect::<Vec<U32Target>>(),
        },
        BigUintTarget {
            limbs: (0..N)
                .into_iter()
                .map(|i| {
                    U32Target::from_target_unsafe(builder.api.select(
                        cmp.into(),
                        iso_3_z_iso_3_a[1].limbs[i].target,
                        denominator_tmp[1].limbs[i].target,
                    ))
                })
                .collect::<Vec<U32Target>>(),
        },
    ];
    let x0 = div_fp2(builder, &numerator, &denominator);
    let x0_2 = mul_fp2(builder, &x0, &x0);
    let x0_3 = mul_fp2(builder, &x0_2, &x0);
    let a_x0 = mul_fp2(builder, &iso_3_a, &x0);
    let x0_3_a_x0 = add_fp2(builder, &x0_3, &a_x0);
    let gx0 = add_fp2(builder, &x0_3_a_x0, &iso_3_b);

    let x1 = mul_fp2(builder, &iso_3_z_t2, &x0);
    let xi3t6 = mul_fp2(builder, &iso_3_z_t2_2, &iso_3_z_t2);
    let gx1 = mul_fp2(builder, &xi3t6, &gx0);

    let is_square = builder.api.add_virtual_bool_target_unsafe();
    let sqrt = [
        builder.api.add_virtual_biguint_target_unsafe(N),
        builder.api.add_virtual_biguint_target_unsafe(N),
    ];

    builder.api.add_simple_generator(SqrtGenerator {
        t: t.clone(),
        x0: gx0.clone(),
        x1: gx1.clone(),
        is_square,
        sqrt: sqrt.clone(),
    });

    builder.api.assert_bool(is_square);
    range_check_fp2(builder, &sqrt);
    let sqrt2 = mul_fp2(builder, &sqrt, &sqrt);
    let gx0_gx1_select = [
        BigUintTarget {
            limbs: (0..N)
                .into_iter()
                .map(|i| {
                    U32Target::from_target_unsafe(builder.api.select(
                        is_square.into(),
                        gx0[0].limbs[i].target,
                        gx1[0].limbs[i].target,
                    ))
                })
                .collect::<Vec<U32Target>>(),
        },
        BigUintTarget {
            limbs: (0..N)
                .into_iter()
                .map(|i| {
                    U32Target::from_target_unsafe(builder.api.select(
                        is_square.into(),
                        gx0[1].limbs[i].target,
                        gx1[1].limbs[i].target,
                    ))
                })
                .collect::<Vec<U32Target>>(),
        },
    ];
    builder.api.connect_biguint(&gx0_gx1_select[0], &sqrt2[0]);
    builder.api.connect_biguint(&gx0_gx1_select[1], &sqrt2[1]);

    let sgn_t = sgn0_fp2(builder, t);
    let sgn_sqrt = sgn0_fp2(builder, &sqrt);
    let sgn_eq = builder.api.is_equal(sgn_t.variable.0, sgn_sqrt.variable.0);
    let sqrt_negate = negate_fp2(builder, &sqrt);
    let y = [
        BigUintTarget {
            limbs: (0..N)
                .into_iter()
                .map(|i| {
                    U32Target::from_target_unsafe(builder.api.select(
                        sgn_eq.into(),
                        sqrt[0].limbs[i].target,
                        sqrt_negate[0].limbs[i].target,
                    ))
                })
                .collect::<Vec<U32Target>>(),
        },
        BigUintTarget {
            limbs: (0..N)
                .into_iter()
                .map(|i| {
                    U32Target::from_target_unsafe(builder.api.select(
                        sgn_eq,
                        sqrt[1].limbs[i].target,
                        sqrt_negate[1].limbs[i].target,
                    ))
                })
                .collect::<Vec<U32Target>>(),
        },
    ];
    let x0_x1_select = [
        BigUintTarget {
            limbs: (0..N)
                .into_iter()
                .map(|i| {
                    U32Target::from_target_unsafe(builder.api.select(
                        is_square,
                        x0[0].limbs[i].target,
                        x1[0].limbs[i].target,
                    ))
                })
                .collect::<Vec<U32Target>>(),
        },
        BigUintTarget {
            limbs: (0..N)
                .into_iter()
                .map(|i| {
                    U32Target::from_target_unsafe(builder.api.select(
                        is_square,
                        x0[1].limbs[i].target,
                        x1[1].limbs[i].target,
                    ))
                })
                .collect::<Vec<U32Target>>(),
        },
    ];

    [x0_x1_select, y]
}

pub fn isogeny_map<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    input: &PointG2Target,
) -> PointG2Target {
    let x = &input[0];
    let x_sq = mul_fp2(builder, x, x);
    let x_cu = mul_fp2(builder, &x_sq, x);

    let coeffs = ISOGENY_COEFFICIENTS_G2
        .iter()
        .map(|c_arr| {
            c_arr
                .iter()
                .map(|c| {
                    let c0 = BigUint::from_str(c[0]).unwrap();
                    let c1 = BigUint::from_str(c[1]).unwrap();
                    [
                        builder.api.constant_biguint(&c0),
                        builder.api.constant_biguint(&c1),
                    ]
                })
                .collect::<Vec<Fp2Target>>()
        })
        .collect::<Vec<Vec<Fp2Target>>>();

    let x_coeffs = mul_fp2(builder, x, &coeffs[0][2]);
    let x_sq_coeffs = mul_fp2(builder, &x_sq, &coeffs[0][1]);
    let x_cu_coeffs = mul_fp2(builder, &x_cu, &coeffs[0][0]);
    let x_num = add_fp2(builder, &coeffs[0][3], &x_coeffs);
    let x_num = add_fp2(builder, &x_num, &x_sq_coeffs);
    let x_num = add_fp2(builder, &x_num, &x_cu_coeffs);

    let x_coeffs = mul_fp2(builder, x, &coeffs[1][2]);
    let x_den = add_fp2(builder, &coeffs[1][3], &x_coeffs);
    let x_den = add_fp2(builder, &x_den, &x_sq);

    let x_coeffs = mul_fp2(builder, x, &coeffs[2][2]);
    let x_sq_coeffs = mul_fp2(builder, &x_sq, &coeffs[2][1]);
    let x_cu_coeffs = mul_fp2(builder, &x_cu, &coeffs[2][0]);
    let y_num = add_fp2(builder, &coeffs[2][3], &x_coeffs);
    let y_num = add_fp2(builder, &y_num, &x_sq_coeffs);
    let y_num = add_fp2(builder, &y_num, &x_cu_coeffs);

    let x_coeffs = mul_fp2(builder, x, &coeffs[3][2]);
    let x_sq_coeffs = mul_fp2(builder, &x_sq, &coeffs[3][1]);
    let y_den = add_fp2(builder, &coeffs[3][3], &x_coeffs);
    let y_den = add_fp2(builder, &y_den, &x_sq_coeffs);
    let y_den = add_fp2(builder, &y_den, &x_cu);

    let x_new = div_fp2(builder, &x_num, &x_den);
    let y_coeff = div_fp2(builder, &y_num, &y_den);
    let y_new = mul_fp2(builder, &input[1], &y_coeff);

    [x_new, y_new]
}

pub fn endomorphism_psi<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    inp: &PointG2Target,
) -> PointG2Target {
    let c0 = [
        builder.api.constant_biguint(&BigUint::from_str("0").unwrap()),
        builder.api.constant_biguint(&BigUint::from_str("4002409555221667392624310435006688643935503118305586438271171395842971157480381377015405980053539358417135540939437").unwrap()),
    ];
    let c1 = [
        builder.api.constant_biguint(&BigUint::from_str("2973677408986561043442465346520108879172042883009249989176415018091420807192182638567116318576472649347015917690530").unwrap()),
        builder.api.constant_biguint(&BigUint::from_str("1028732146235106349975324479215795277384839936929757896155643118032610843298655225875571310552543014690878354869257").unwrap()),
    ];
    let frob = [
        frobenius_map(builder, &inp[0], 1),
        frobenius_map(builder, &inp[1], 1),
    ];
    [
        mul_fp2(builder, &c0, &frob[0]),
        mul_fp2(builder, &c1, &frob[1]),
    ]
}

pub fn endomorphism_psi2<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    inp: &PointG2Target,
) -> PointG2Target {
    let c = builder.api.constant_biguint(&BigUint::from_str("4002409555221667392624310435006688643935503118305586438271171395842971157480381377015405980053539358417135540939436").unwrap());
    [
        [
            mul_fp(builder, &inp[0][0], &c),
            mul_fp(builder, &inp[0][1], &c),
        ],
        negate_fp2(builder, &inp[1]),
    ]
}

fn mul_curve_x<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    point: &PointG2Target,
) -> PointG2Target {
    let mut temp_x = BLS_X;
    let mut temp_point_d = point.clone();
    let mut point_r = point.clone();
    while temp_x > 0 {
        if temp_x & 1 == 1 {
            point_r = g2_add_without_generator(builder, &point_r, &temp_point_d);
        }
        temp_point_d = g2_add_without_generator(builder, &temp_point_d, &temp_point_d);
        temp_x >>= 1;
    }

    point_r
}

pub fn my_clear_cofactor_g2<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    point: &PointG2Target,
) -> PointG2Target {
    let t1 = mul_curve_x(builder, &point); // [-x]P
                                           // Is the endomorphism fine?
    let mut t2 = endomorphism_psi(builder, &point); // Ψ(P)
    let mut t3 = g2_add_without_generator(builder, &point, &point); // 2P
                                                                    // is the squared endomorphism fine?
    t3 = endomorphism_psi2(builder, &t3); // Ψ²(2P)
    let t2_neg = g2_negate(builder, &t2); // - Ψ(P)
    t3 = g2_add_without_generator(builder, &t3, &t2_neg); // Ψ²(2P) - Ψ(P)
    t2 = g2_add_without_generator(builder, &t1, &t2); // [-x]P + Ψ(P)
    t2 = mul_curve_x(builder, &t2); // [x²]P - [x]Ψ(P)
    t3 = g2_add_without_generator(builder, &t3, &t2); // Ψ²(2P) - Ψ(P) + [x²]P - [x]Ψ(P)
    let t1_neg = g2_negate(builder, &t1); // - [-x]P
    t3 = g2_add_without_generator(builder, &t3, &t1_neg); // Ψ²(2P) - Ψ(P) + [x²]P - [x]Ψ(P) + [x]P
    let point_neg = g2_negate(builder, &point); // - (P)
    let q = g2_add_without_generator(builder, &t3, &point_neg); // Ψ²(2P) - Ψ(P) + [x²]P - [x]Ψ(P) + [x]P - 1P =>

    q // [x²-x-1]P + [x-1]Ψ(P) + Ψ²(2P)
}

pub fn clear_cofactor_g2<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    inp: &PointG2Target,
) -> PointG2Target {
    let a = [
        builder
            .api
            .constant_biguint(&BigUint::from_str("0").unwrap()),
        builder
            .api
            .constant_biguint(&BigUint::from_str("0").unwrap()),
    ];
    let b = [
        builder
            .api
            .constant_biguint(&BigUint::from_str("4").unwrap()),
        builder
            .api
            .constant_biguint(&BigUint::from_str("4").unwrap()),
    ];

    //
    let clear_cofactor_y1 = builder
        .api
        .constant_biguint(&BigUint::from_str("4").unwrap());
    let r = builder.api.is_equal_biguint(&clear_cofactor_y1, &b[0]);
    builder.api.assert_one(r.target);
    let r = builder.api.is_equal_biguint(&clear_cofactor_y1, &b[1]);
    builder.api.assert_one(r.target);

    //

    let fals = builder._false();
    let x_p = g2_scalar_mul(builder, inp, &b); // Pb where b = (x₁, x₂+i) = 4, (y₁, y₂+i) = 4
    let psi_p = endomorphism_psi(builder, inp); // Ψ(P)
    let neg_p = g2_negate(builder, &inp); // -P
    let neg_psi_p = g2_negate(builder, &psi_p); // -Ψ(P)
    let double_p = g2_double(builder, &inp, &a, &b); // 2P
    let psi2_2p = endomorphism_psi2(builder, &double_p); // Ψ²(2P)

    // Pb + P + a + b
    let add0 = g2_add(builder, &x_p, fals.into(), &inp, fals.into(), &a, &b);
    // Pb + P + a + b + -Ψ(P) + a + b
    let add1 = g2_add(builder, &add0, fals.into(), &neg_psi_p, fals.into(), &a, &b);
    // (Pb + P + a + b + -Ψ(P) + a + b)b
    let x_add1 = g2_scalar_mul(builder, &add1, &b);
    let add2 = g2_add(builder, &x_add1, fals.into(), &neg_p, fals.into(), &a, &b);
    let add3 = g2_add(builder, &add2, fals.into(), &neg_psi_p, fals.into(), &a, &b);
    let add4 = g2_add(builder, &add3, fals.into(), &psi2_2p, fals.into(), &a, &b);

    add4
}

pub fn hash_to_curve<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    msg: &[Variable],
) -> PointG2Target {
    let iso_3_a = [
        builder.api.constant_biguint(&0.to_biguint().unwrap()),
        builder.api.constant_biguint(&240.to_biguint().unwrap()),
    ];
    let iso_3_b = [
        builder.api.constant_biguint(&1012.to_biguint().unwrap()),
        builder.api.constant_biguint(&1012.to_biguint().unwrap()),
    ];

    let u = hash_to_field(builder, msg, 2);
    let pt1 = map_to_curve_simple_swu_9mod16(builder, &u[0]);
    let pt2 = map_to_curve_simple_swu_9mod16(builder, &u[1]);
    let no = builder._false();
    let pt1pt2 = g2_add(
        builder,
        &pt1,
        no.into(),
        &pt2,
        no.into(),
        &iso_3_a,
        &iso_3_b,
    );
    let isogeny_mapping = isogeny_map(builder, &pt1pt2);
    let clear_cofactor = clear_cofactor_g2(builder, &isogeny_mapping);

    clear_cofactor
}

#[derive(Debug, Default)]
pub struct SqrtGenerator {
    t: Fp2Target,
    x0: Fp2Target,
    x1: Fp2Target,
    is_square: BoolTarget,
    sqrt: Fp2Target,
}

impl<F: RichField + Extendable<D>, const D: usize> SimpleGenerator<F, D> for SqrtGenerator {
    fn id(&self) -> String {
        "Fp2SqrtGenerator".to_string()
    }

    fn dependencies(&self) -> Vec<Target> {
        self.t
            .iter()
            .chain(self.x0.iter().chain(self.x1.iter()))
            .flat_map(|f| f.limbs.iter().map(|l| l.target))
            .collect::<Vec<Target>>()
    }

    fn run_once(&self, witness: &PartitionWitness<F>, out_buffer: &mut GeneratedValues<F>) {
        let x0_c0 = witness.get_biguint_target(self.x0[0].clone());
        let x0_c1 = witness.get_biguint_target(self.x0[1].clone());

        let x0_fp2 = Fp2([
            Fp::get_fp_from_biguint(x0_c0),
            Fp::get_fp_from_biguint(x0_c1),
        ]);
        let p2_7_16 = (modulus().pow(2) + 7u32) / 16u32;
        let sqrt_candidate = x0_fp2.pow(Fp2::one(), p2_7_16);
        let roots = Fp2::roots_of_unity_8th();
        let mut is_square = false;
        let mut sqrt_witness = Fp2::zero();
        for root in roots {
            let sqrt_tmp = sqrt_candidate * root;
            if sqrt_tmp * sqrt_tmp == x0_fp2 {
                is_square = true;
                sqrt_witness = sqrt_tmp;
                break;
            }
        }
        out_buffer.set_bool_target(self.is_square, is_square);
        if is_square {
            out_buffer.set_biguint_target(&self.sqrt[0], &sqrt_witness.0[0].to_biguint());
            out_buffer.set_biguint_target(&self.sqrt[1], &sqrt_witness.0[1].to_biguint());
            return;
        }

        let t_c0 = witness.get_biguint_target(self.t[0].clone());
        let t_c1 = witness.get_biguint_target(self.t[1].clone());
        let t_fp2 = Fp2([Fp::get_fp_from_biguint(t_c0), Fp::get_fp_from_biguint(t_c1)]);

        let x1_c0 = witness.get_biguint_target(self.x1[0].clone());
        let x1_c1 = witness.get_biguint_target(self.x1[1].clone());
        let x1_fp2 = Fp2([
            Fp::get_fp_from_biguint(x1_c0),
            Fp::get_fp_from_biguint(x1_c1),
        ]);

        let t3 = t_fp2 * t_fp2 * t_fp2;
        let sqrt_candidate = sqrt_candidate * t3;
        let etas = Fp2::etas();
        let mut is_square1 = false;
        for eta in etas {
            let sqrt_tmp = sqrt_candidate * eta;
            if sqrt_tmp * sqrt_tmp == x1_fp2 {
                is_square1 = true;
                sqrt_witness = sqrt_tmp;
                break;
            }
        }
        assert!(is_square1, "Failed in square root generator");
        out_buffer.set_biguint_target(&self.sqrt[0], &sqrt_witness.0[0].to_biguint());
        out_buffer.set_biguint_target(&self.sqrt[1], &sqrt_witness.0[1].to_biguint());
    }

    fn serialize(&self, dst: &mut Vec<u8>, _common_data: &CommonCircuitData<F, D>) -> IoResult<()> {
        self.t[0].serialize(dst)?;
        self.t[1].serialize(dst)?;
        self.x0[0].serialize(dst)?;
        self.x0[1].serialize(dst)?;
        self.x1[0].serialize(dst)?;
        self.x1[1].serialize(dst)?;
        dst.write_target_bool(self.is_square)?;
        self.sqrt[0].serialize(dst)?;
        self.sqrt[1].serialize(dst)
    }

    fn deserialize(src: &mut Buffer, _common_data: &CommonCircuitData<F, D>) -> IoResult<Self>
    where
        Self: Sized,
    {
        let t_c0 = BigUintTarget::deserialize(src)?;
        let t_c1 = BigUintTarget::deserialize(src)?;
        let x0_c0 = BigUintTarget::deserialize(src)?;
        let x0_c1 = BigUintTarget::deserialize(src)?;
        let x1_c0 = BigUintTarget::deserialize(src)?;
        let x1_c1 = BigUintTarget::deserialize(src)?;
        let is_square = src.read_target_bool()?;
        let sqrt_c0 = BigUintTarget::deserialize(src)?;
        let sqrt_c1 = BigUintTarget::deserialize(src)?;
        Ok(Self {
            t: [t_c0, t_c1],
            x0: [x0_c0, x0_c1],
            x1: [x1_c0, x1_c1],
            is_square,
            sqrt: [sqrt_c0, sqrt_c1],
        })
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use itertools::Itertools;
    use num_bigint::BigUint;
    use plonky2::field::{goldilocks_field::GoldilocksField, types::Field64};
    use plonky2x::frontend::{
        builder::DefaultBuilder,
        uint::num::biguint::CircuitBuilderBiguint,
        vars::{CircuitVariable, Variable},
    };

    use crate::verification::aggregation::hash_to_curve::map_to_curve_simple_swu_9mod16;

    use super::{hash_to_curve, isogeny_map, my_clear_cofactor_g2};

    #[test]
    fn test_hash_to_curve() {
        let mut builder = DefaultBuilder::new();
        let msg = vec![
            Variable::constant(&mut builder, GoldilocksField(9636)),
            Variable::constant(&mut builder, GoldilocksField(8499)),
            Variable::constant(&mut builder, GoldilocksField(980)),
            Variable::constant(&mut builder, GoldilocksField(3289)),
            Variable::constant(&mut builder, GoldilocksField(2380)),
            Variable::constant(&mut builder, GoldilocksField(4091)),
            Variable::constant(&mut builder, GoldilocksField(4494)),
            Variable::constant(&mut builder, GoldilocksField(7841)),
            Variable::constant(&mut builder, GoldilocksField(8175)),
            Variable::constant(&mut builder, GoldilocksField(1645)),
            Variable::constant(&mut builder, GoldilocksField(9486)),
            Variable::constant(&mut builder, GoldilocksField(6069)),
            Variable::constant(&mut builder, GoldilocksField(8507)),
            Variable::constant(&mut builder, GoldilocksField(739)),
            Variable::constant(&mut builder, GoldilocksField(4264)),
            Variable::constant(&mut builder, GoldilocksField(209)),
            Variable::constant(&mut builder, GoldilocksField(1174)),
            Variable::constant(&mut builder, GoldilocksField(7352)),
            Variable::constant(&mut builder, GoldilocksField(1824)),
            Variable::constant(&mut builder, GoldilocksField(5981)),
            Variable::constant(&mut builder, GoldilocksField(3557)),
            Variable::constant(&mut builder, GoldilocksField(8703)),
            Variable::constant(&mut builder, GoldilocksField(368)),
            Variable::constant(&mut builder, GoldilocksField(9610)),
            Variable::constant(&mut builder, GoldilocksField(6902)),
            Variable::constant(&mut builder, GoldilocksField(3)),
        ];
        let hash_to_curve_res = hash_to_curve(&mut builder, &msg);

        let exp_clear_cofactor_x0 = builder.api.constant_biguint(&BigUint::from_str("3898314311143498598232928636302843201147417323239224718360789834030193898102380674004641739485821762063383300863223").unwrap());
        let exp_clear_cofactor_x1 = builder.api.constant_biguint(&BigUint::from_str("1907381618300654678833809042530528045219202973036460400592647259752361578155388048783374146362885595712785322935889").unwrap());
        let exp_clear_cofactor_y0 = builder.api.constant_biguint(&BigUint::from_str("2533497754358129344573819271980775177420433434576664259373048380327090716743523684037647313977831493346790338697416").unwrap());
        let exp_clear_cofactor_y1 = builder.api.constant_biguint(&BigUint::from_str("2476458800839415772166412841480638992289141020062264369481606378598402067179861075321658473879343813706964824257238").unwrap());
        let r = builder
            .api
            .is_equal_biguint(&exp_clear_cofactor_x0, &hash_to_curve_res[0][0]);
        builder.api.assert_one(r.target);
        let r = builder
            .api
            .is_equal_biguint(&exp_clear_cofactor_x1, &hash_to_curve_res[0][1]);
        builder.api.assert_one(r.target);
        let r = builder
            .api
            .is_equal_biguint(&exp_clear_cofactor_y0, &hash_to_curve_res[1][0]);
        builder.api.assert_one(r.target);
        let r = builder
            .api
            .is_equal_biguint(&exp_clear_cofactor_y1, &hash_to_curve_res[1][1]);
        builder.api.assert_one(r.target);

        println!("sled tova");

        // Define your circuit.
        let mut res_output: Vec<GoldilocksField> = Vec::new();
        for i in 0..hash_to_curve_res.len() {
            for j in 0..hash_to_curve_res[i].len() {
                for k in 0..hash_to_curve_res[i][j].limbs.len() {
                    builder.write(Variable(hash_to_curve_res[i][j].limbs[k].target));
                }
            }
        }

        println!("dali stiga do tuk 1?");

        // Build your circuit.
        let circuit = builder.build();
        println!("dali stiga do tuk 1.1?");

        // Write to the circuit input.
        let input = circuit.input();
        println!("dali stiga do tuk 1.2?");

        // Generate a proof.
        let (proof, mut output) = circuit.prove(&input);
        println!("dali stiga do tuk 1.5?");
        // Verify proof.
        circuit.verify(&proof, &input, &output);

        // Read output.
        for i in 0..hash_to_curve_res.len() {
            for j in 0..hash_to_curve_res[i].len() {
                for _ in 0..hash_to_curve_res[i][j].limbs.len() {
                    res_output.push(output.read::<Variable>())
                }
            }
        }

        println!("dali stiga do tuk 2?");

        let mut biguint_res: Vec<BigUint> = Vec::new();
        for i in 0..4 {
            biguint_res.push(BigUint::new(
                res_output[(i * 12)..(i * 12) + 12]
                    .iter()
                    .map(|f| (f.0 % GoldilocksField::ORDER) as u32)
                    .collect_vec(),
            ));
        }

        let _expected_biguint_targets = vec![
                    BigUint::from_str("3898314311143498598232928636302843201147417323239224718360789834030193898102380674004641739485821762063383300863223").unwrap(), 
                    BigUint::from_str("1907381618300654678833809042530528045219202973036460400592647259752361578155388048783374146362885595712785322935889").unwrap(), 
                    BigUint::from_str("2533497754358129344573819271980775177420433434576664259373048380327090716743523684037647313977831493346790338697416").unwrap(), 
                    BigUint::from_str("2476458800839415772166412841480638992289141020062264369481606378598402067179861075321658473879343813706964824257238").unwrap()
                ];

        println!("dali stiga do tuk 3?");

        for i in 0..4 {
            println!("curr: {:?}", biguint_res[i]);
            //assert_eq!(biguint_res[i], expected_biguint_targets[i]);
        }
    }

    #[test]
    fn test_map_to_curve_simple_swu_9mod16() {
        let mut builder = DefaultBuilder::new();
        let x = [builder.api.constant_biguint(&BigUint::from_str("474682481268733588266168000983897038833463740369371343293271315606510847229825856506681723856424762498931536081381").unwrap()), builder.api.constant_biguint(&BigUint::from_str("1366297191634768530389324840135632614622170346303255080801396974208665528754948924260000453159829725659141010218083").unwrap())];
        let new_point = map_to_curve_simple_swu_9mod16(&mut builder, &x);

        // Define your circuit.
        let mut res_output: Vec<GoldilocksField> = Vec::new();
        for i in 0..new_point.len() {
            for j in 0..new_point[i].len() {
                for k in 0..new_point[i][j].limbs.len() {
                    builder.write(Variable(new_point[i][j].limbs[k].target));
                }
            }
        }

        // Build your circuit.
        let circuit = builder.build();

        // Write to the circuit input.
        let input = circuit.input();

        // Generate a proof.
        let (proof, mut output) = circuit.prove(&input);
        // Verify proof.
        circuit.verify(&proof, &input, &output);

        // Read output.
        for i in 0..new_point.len() {
            for j in 0..new_point[i].len() {
                for _ in 0..new_point[i][j].limbs.len() {
                    res_output.push(output.read::<Variable>())
                }
            }
        }

        let mut biguint_res: Vec<BigUint> = Vec::new();

        for i in 0..4 {
            biguint_res.push(BigUint::new(
                res_output[(i * 12)..((i * 12) + 12)]
                    .iter()
                    .map(|f| ((f.0 % GoldilocksField::ORDER) as u32))
                    .collect_vec(),
            ));
        }

        let expected_biguint_targets = vec![
            BigUint::from_str("3060844272194546509744375366937392691364803424242981321948532731206236794105714573248676325992693995641546323869947").unwrap(), 
            BigUint::from_str("2178088723896136927227615444202612183719092972593095669593917181168791652031398769747908182877951150253834691003695").unwrap(), 
            BigUint::from_str("2414062066557001374784906001337739211138362843766395178252280511119838997923178981557780591344278921569184403008099").unwrap(), 
            BigUint::from_str("902142789549649010950853691727709369432566981811071618377331254273490164668206477123333794980363358097421619541372").unwrap()
        ];

        for i in 0..4 {
            assert_eq!(biguint_res[i], expected_biguint_targets[i]);
        }
    }

    #[test]
    fn test_isogeny_map() {
        let mut builder = DefaultBuilder::new();
        let x = [builder.api.constant_biguint(&BigUint::from_str("474682481268733588266168000983897038833463740369371343293271315606510847229825856506681723856424762498931536081381").unwrap()), builder.api.constant_biguint(&BigUint::from_str("1366297191634768530389324840135632614622170346303255080801396974208665528754948924260000453159829725659141010218083").unwrap())];
        let new_point = map_to_curve_simple_swu_9mod16(&mut builder, &x);
        let iso_map_r = isogeny_map(&mut builder, &new_point);

        let iso_map_r_x0 = builder.api.constant_biguint(&BigUint::from_str("3020098988166152265957458699713409264776064412968511868273334310978607420463777702053743668373252848938048859569472").unwrap());
        let iso_map_r_x1 = builder.api.constant_biguint(&BigUint::from_str("1458981974613365650201781947361855472098362440235925030682710979747620221343697516696212172566912716109989777361662").unwrap());
        let iso_map_r_y0 = builder.api.constant_biguint(&BigUint::from_str("1834291692231285600047846672091248684005847013394827595644756391313325861691761060706376473203409023894171500990751").unwrap());
        let iso_map_r_y1 = builder.api.constant_biguint(&BigUint::from_str("2613278682710607327768853275311538731542148746765923401506548661907721927393566272464025106984186092820519334410455").unwrap());
        let r = builder
            .api
            .is_equal_biguint(&iso_map_r_x0, &iso_map_r[0][0]);
        builder.api.assert_one(r.target);
        let r = builder
            .api
            .is_equal_biguint(&iso_map_r_x1, &iso_map_r[0][1]);
        builder.api.assert_one(r.target);
        let r = builder
            .api
            .is_equal_biguint(&iso_map_r_y0, &iso_map_r[1][0]);
        builder.api.assert_one(r.target);
        let r = builder
            .api
            .is_equal_biguint(&iso_map_r_y1, &iso_map_r[1][1]);
        builder.api.assert_one(r.target);

        // Define your circuit.
        let mut res_output: Vec<GoldilocksField> = Vec::new();
        for i in 0..iso_map_r.len() {
            for j in 0..iso_map_r[i].len() {
                for k in 0..iso_map_r[i][j].limbs.len() {
                    builder.write(Variable(iso_map_r[i][j].limbs[k].target));
                }
            }
        }

        // Build your circuit.
        let circuit = builder.build();

        // Write to the circuit input.
        let input = circuit.input();

        // Generate a proof.
        let (proof, mut output) = circuit.prove(&input);
        // Verify proof.
        circuit.verify(&proof, &input, &output);

        // Read output.
        for i in 0..iso_map_r.len() {
            for j in 0..iso_map_r[i].len() {
                for _ in 0..iso_map_r[i][j].limbs.len() {
                    res_output.push(output.read::<Variable>())
                }
            }
        }

        let mut biguint_res: Vec<BigUint> = Vec::new();

        for i in 0..4 {
            biguint_res.push(BigUint::new(
                res_output[(i * 12)..((i * 12) + 12)]
                    .iter()
                    .map(|f| ((f.0 % GoldilocksField::ORDER) as u32))
                    .collect_vec(),
            ));
        }

        let expected_biguint_targets = vec![
            BigUint::from_str("3020098988166152265957458699713409264776064412968511868273334310978607420463777702053743668373252848938048859569472").unwrap(),
            BigUint::from_str("1458981974613365650201781947361855472098362440235925030682710979747620221343697516696212172566912716109989777361662").unwrap(),
            BigUint::from_str("1834291692231285600047846672091248684005847013394827595644756391313325861691761060706376473203409023894171500990751").unwrap(),
            BigUint::from_str("2613278682710607327768853275311538731542148746765923401506548661907721927393566272464025106984186092820519334410455").unwrap()
        ];

        for i in 0..4 {
            println!("biguint_res[i] is: {:?}", biguint_res[i]);
        }

        for i in 0..4 {
            assert_eq!(biguint_res[i], expected_biguint_targets[i]);
        }
    }

    #[test]
    fn test_clear_cofactor() {
        let mut builder = DefaultBuilder::new();
        let x = [builder.api.constant_biguint(&BigUint::from_str("474682481268733588266168000983897038833463740369371343293271315606510847229825856506681723856424762498931536081381").unwrap()), builder.api.constant_biguint(&BigUint::from_str("1366297191634768530389324840135632614622170346303255080801396974208665528754948924260000453159829725659141010218083").unwrap())];
        let new_point = map_to_curve_simple_swu_9mod16(&mut builder, &x);
        let iso_map_r = isogeny_map(&mut builder, &new_point);
        let clear_cofactor = my_clear_cofactor_g2(&mut builder, &iso_map_r);

        // let clear_cofactor_x0 = builder
        //     .api
        //     .constant_biguint(&BigUint::from_str("3898314311143498598232928636302843201147417323239224718360789834030193898102380674004641739485821762063383300863223").unwrap());
        // let clear_cofactor_x1 = builder
        //     .api
        //     .constant_biguint(&BigUint::from_str("1907381618300654678833809042530528045219202973036460400592647259752361578155388048783374146362885595712785322935889").unwrap());
        // let clear_cofactor_y0 = builder
        //     .api
        //     .constant_biguint(&BigUint::from_str("2533497754358129344573819271980775177420433434576664259373048380327090716743523684037647313977831493346790338697416").unwrap());
        // let clear_cofactor_y1 = builder
        //     .api
        //     .constant_biguint(&BigUint::from_str("2476458800839415772166412841480638992289141020062264369481606378598402067179861075321658473879343813706964824257238").unwrap());
        // let r = builder
        //     .api
        //     .is_equal_biguint(&clear_cofactor_x0, &clear_cofactor[0][0]);
        // builder.api.assert_one(r.target);
        // let r = builder
        //     .api
        //     .is_equal_biguint(&clear_cofactor_x1, &clear_cofactor[0][1]);
        // builder.api.assert_one(r.target);
        // let r = builder
        //     .api
        //     .is_equal_biguint(&clear_cofactor_y0, &clear_cofactor[1][0]);
        // builder.api.assert_one(r.target);
        // let r = builder
        //     .api
        //     .is_equal_biguint(&clear_cofactor_y1, &clear_cofactor[1][1]);
        // builder.api.assert_one(r.target);

        // Define your circuit.
        let mut res_output: Vec<GoldilocksField> = Vec::new();
        for i in 0..clear_cofactor.len() {
            for j in 0..clear_cofactor[i].len() {
                for k in 0..clear_cofactor[i][j].limbs.len() {
                    builder.write(Variable(clear_cofactor[i][j].limbs[k].target));
                }
            }
        }

        // Build your circuit.
        let circuit = builder.build();

        // Write to the circuit input.
        let input = circuit.input();

        // Generate a proof.
        let (proof, mut output) = circuit.prove(&input);

        // Verify proof.
        circuit.verify(&proof, &input, &output);

        // Read output.
        for i in 0..clear_cofactor.len() {
            for j in 0..clear_cofactor[i].len() {
                for _ in 0..clear_cofactor[i][j].limbs.len() {
                    res_output.push(output.read::<Variable>())
                }
            }
        }

        let mut biguint_res: Vec<BigUint> = Vec::new();

        for i in 0..4 {
            biguint_res.push(BigUint::new(
                res_output[(i * 12)..((i * 12) + 12)]
                    .iter()
                    .map(|f| ((f.0 % GoldilocksField::ORDER) as u32))
                    .collect_vec(),
            ));
        }

        let expected_biguint_targets = vec![
            BigUint::from_str("3898314311143498598232928636302843201147417323239224718360789834030193898102380674004641739485821762063383300863223").unwrap(),
            BigUint::from_str("1907381618300654678833809042530528045219202973036460400592647259752361578155388048783374146362885595712785322935889").unwrap(),
            BigUint::from_str("2533497754358129344573819271980775177420433434576664259373048380327090716743523684037647313977831493346790338697416").unwrap(),
            BigUint::from_str("2476458800839415772166412841480638992289141020062264369481606378598402067179861075321658473879343813706964824257238").unwrap()
        ];

        for i in 0..4 {
            println!("biguint_res[i] is: {:?}", biguint_res[i]);
        }

        for i in 0..4 {
            assert_eq!(biguint_res[i], expected_biguint_targets[i]);
        }
    }
}
