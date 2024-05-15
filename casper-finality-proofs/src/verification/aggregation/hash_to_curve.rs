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
    curves::g2::{g2_add, g2_double, g2_negate, g2_scalar_mul, PointG2Target},
    fields::{
        fp::{mul_fp, LIMBS},
        fp2::{
            add_fp2, div_fp2, frobenius_map, is_zero, mul_fp2, negate_fp2, range_check_fp2,
            sgn0_fp2, Fp2Target,
        },
    },
    utils::native_bls::{modulus, Fp, Fp2, Pow},
};

use super::hash_to_field::hash_to_field;

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
            limbs: (0..LIMBS)
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
            limbs: (0..LIMBS)
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
        builder.api.add_virtual_biguint_target_unsafe(LIMBS),
        builder.api.add_virtual_biguint_target_unsafe(LIMBS),
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
            limbs: (0..LIMBS)
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
            limbs: (0..LIMBS)
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
            limbs: (0..LIMBS)
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
            limbs: (0..LIMBS)
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
            limbs: (0..LIMBS)
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
            limbs: (0..LIMBS)
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
    let fals = builder._false();
    let x_p = g2_scalar_mul(builder, inp, &b);
    let psi_p = endomorphism_psi(builder, inp);
    let neg_p = g2_negate(builder, &inp);
    let neg_psi_p = g2_negate(builder, &psi_p);
    let double_p = g2_double(builder, &inp, &a, &b);
    let psi2_2p = endomorphism_psi2(builder, &double_p);

    let add0 = g2_add(builder, &x_p, fals, &inp, fals, &a, &b);
    let add1 = g2_add(builder, &add0, fals, &neg_psi_p, fals, &a, &b);
    let x_add1 = g2_scalar_mul(builder, &add1, &b);
    let add2 = g2_add(builder, &x_add1, fals, &neg_p, fals, &a, &b);
    let add3 = g2_add(builder, &add2, fals, &neg_psi_p, fals, &a, &b);
    let add4 = g2_add(builder, &add3, fals, &psi2_2p, fals, &a, &b);
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

    use crate::verification::{
        aggregation::hash_to_curve::map_to_curve_simple_swu_9mod16, fields::fp::LIMBS,
    };

    use super::{clear_cofactor_g2, hash_to_curve, isogeny_map};

    #[test]
    fn test_hash_to_curve() {
        let mut builder = DefaultBuilder::new();
        let msg = vec![
            Variable::constant(&mut builder, GoldilocksField(103)),
            Variable::constant(&mut builder, GoldilocksField(140)),
            Variable::constant(&mut builder, GoldilocksField(163)),
            Variable::constant(&mut builder, GoldilocksField(210)),
            Variable::constant(&mut builder, GoldilocksField(238)),
            Variable::constant(&mut builder, GoldilocksField(252)),
            Variable::constant(&mut builder, GoldilocksField(75)),
            Variable::constant(&mut builder, GoldilocksField(8)),
            Variable::constant(&mut builder, GoldilocksField(227)),
            Variable::constant(&mut builder, GoldilocksField(27)),
            Variable::constant(&mut builder, GoldilocksField(60)),
            Variable::constant(&mut builder, GoldilocksField(229)),
            Variable::constant(&mut builder, GoldilocksField(125)),
            Variable::constant(&mut builder, GoldilocksField(150)),
            Variable::constant(&mut builder, GoldilocksField(241)),
            Variable::constant(&mut builder, GoldilocksField(222)),
            Variable::constant(&mut builder, GoldilocksField(217)),
            Variable::constant(&mut builder, GoldilocksField(156)),
            Variable::constant(&mut builder, GoldilocksField(178)),
            Variable::constant(&mut builder, GoldilocksField(17)),
            Variable::constant(&mut builder, GoldilocksField(14)),
            Variable::constant(&mut builder, GoldilocksField(199)),
            Variable::constant(&mut builder, GoldilocksField(15)),
            Variable::constant(&mut builder, GoldilocksField(172)),
            Variable::constant(&mut builder, GoldilocksField(94)),
            Variable::constant(&mut builder, GoldilocksField(179)),
            Variable::constant(&mut builder, GoldilocksField(249)),
            Variable::constant(&mut builder, GoldilocksField(0)),
            Variable::constant(&mut builder, GoldilocksField(197)),
            Variable::constant(&mut builder, GoldilocksField(206)),
            Variable::constant(&mut builder, GoldilocksField(104)),
            Variable::constant(&mut builder, GoldilocksField(200)),
            Variable::constant(&mut builder, GoldilocksField(165)),
            Variable::constant(&mut builder, GoldilocksField(253)),
            Variable::constant(&mut builder, GoldilocksField(55)),
            Variable::constant(&mut builder, GoldilocksField(147)),
            Variable::constant(&mut builder, GoldilocksField(171)),
            Variable::constant(&mut builder, GoldilocksField(191)),
            Variable::constant(&mut builder, GoldilocksField(118)),
            Variable::constant(&mut builder, GoldilocksField(189)),
            Variable::constant(&mut builder, GoldilocksField(133)),
            Variable::constant(&mut builder, GoldilocksField(138)),
            Variable::constant(&mut builder, GoldilocksField(2)),
            Variable::constant(&mut builder, GoldilocksField(22)),
            Variable::constant(&mut builder, GoldilocksField(237)),
            Variable::constant(&mut builder, GoldilocksField(6)),
            Variable::constant(&mut builder, GoldilocksField(62)),
            Variable::constant(&mut builder, GoldilocksField(10)),
            Variable::constant(&mut builder, GoldilocksField(68)),
            Variable::constant(&mut builder, GoldilocksField(105)),
            Variable::constant(&mut builder, GoldilocksField(208)),
            Variable::constant(&mut builder, GoldilocksField(102)),
            Variable::constant(&mut builder, GoldilocksField(66)),
            Variable::constant(&mut builder, GoldilocksField(70)),
            Variable::constant(&mut builder, GoldilocksField(170)),
            Variable::constant(&mut builder, GoldilocksField(114)),
            Variable::constant(&mut builder, GoldilocksField(194)),
            Variable::constant(&mut builder, GoldilocksField(80)),
            Variable::constant(&mut builder, GoldilocksField(215)),
            Variable::constant(&mut builder, GoldilocksField(5)),
            Variable::constant(&mut builder, GoldilocksField(63)),
            Variable::constant(&mut builder, GoldilocksField(95)),
            Variable::constant(&mut builder, GoldilocksField(202)),
            Variable::constant(&mut builder, GoldilocksField(1)),
            Variable::constant(&mut builder, GoldilocksField(99)),
            Variable::constant(&mut builder, GoldilocksField(153)),
            Variable::constant(&mut builder, GoldilocksField(67)),
            Variable::constant(&mut builder, GoldilocksField(115)),
            Variable::constant(&mut builder, GoldilocksField(7)),
            Variable::constant(&mut builder, GoldilocksField(122)),
            Variable::constant(&mut builder, GoldilocksField(235)),
            Variable::constant(&mut builder, GoldilocksField(255)),
            Variable::constant(&mut builder, GoldilocksField(142)),
            Variable::constant(&mut builder, GoldilocksField(44)),
            Variable::constant(&mut builder, GoldilocksField(3)),
            Variable::constant(&mut builder, GoldilocksField(65)),
            Variable::constant(&mut builder, GoldilocksField(190)),
            Variable::constant(&mut builder, GoldilocksField(166)),
            Variable::constant(&mut builder, GoldilocksField(218)),
            Variable::constant(&mut builder, GoldilocksField(72)),
            Variable::constant(&mut builder, GoldilocksField(230)),
            Variable::constant(&mut builder, GoldilocksField(196)),
            Variable::constant(&mut builder, GoldilocksField(24)),
            Variable::constant(&mut builder, GoldilocksField(88)),
            Variable::constant(&mut builder, GoldilocksField(146)),
            Variable::constant(&mut builder, GoldilocksField(193)),
            Variable::constant(&mut builder, GoldilocksField(211)),
            Variable::constant(&mut builder, GoldilocksField(90)),
            Variable::constant(&mut builder, GoldilocksField(37)),
            Variable::constant(&mut builder, GoldilocksField(173)),
            Variable::constant(&mut builder, GoldilocksField(71)),
            Variable::constant(&mut builder, GoldilocksField(152)),
            Variable::constant(&mut builder, GoldilocksField(21)),
            Variable::constant(&mut builder, GoldilocksField(226)),
            Variable::constant(&mut builder, GoldilocksField(89)),
            Variable::constant(&mut builder, GoldilocksField(79)),
            Variable::constant(&mut builder, GoldilocksField(239)),
            Variable::constant(&mut builder, GoldilocksField(81)),
            Variable::constant(&mut builder, GoldilocksField(149)),
            Variable::constant(&mut builder, GoldilocksField(135)),
            Variable::constant(&mut builder, GoldilocksField(188)),
            Variable::constant(&mut builder, GoldilocksField(51)),
            Variable::constant(&mut builder, GoldilocksField(52)),
            Variable::constant(&mut builder, GoldilocksField(116)),
            Variable::constant(&mut builder, GoldilocksField(26)),
            Variable::constant(&mut builder, GoldilocksField(30)),
            Variable::constant(&mut builder, GoldilocksField(126)),
            Variable::constant(&mut builder, GoldilocksField(31)),
            Variable::constant(&mut builder, GoldilocksField(35)),
            Variable::constant(&mut builder, GoldilocksField(240)),
            Variable::constant(&mut builder, GoldilocksField(201)),
            Variable::constant(&mut builder, GoldilocksField(101)),
            Variable::constant(&mut builder, GoldilocksField(33)),
            Variable::constant(&mut builder, GoldilocksField(61)),
            Variable::constant(&mut builder, GoldilocksField(220)),
            Variable::constant(&mut builder, GoldilocksField(192)),
            Variable::constant(&mut builder, GoldilocksField(86)),
            Variable::constant(&mut builder, GoldilocksField(47)),
            Variable::constant(&mut builder, GoldilocksField(214)),
            Variable::constant(&mut builder, GoldilocksField(243)),
            Variable::constant(&mut builder, GoldilocksField(224)),
            Variable::constant(&mut builder, GoldilocksField(136)),
            Variable::constant(&mut builder, GoldilocksField(50)),
            Variable::constant(&mut builder, GoldilocksField(56)),
            Variable::constant(&mut builder, GoldilocksField(42)),
            Variable::constant(&mut builder, GoldilocksField(233)),
            Variable::constant(&mut builder, GoldilocksField(148)),
            Variable::constant(&mut builder, GoldilocksField(244)),
            Variable::constant(&mut builder, GoldilocksField(203)),
            Variable::constant(&mut builder, GoldilocksField(198)),
            Variable::constant(&mut builder, GoldilocksField(195)),
            Variable::constant(&mut builder, GoldilocksField(120)),
            Variable::constant(&mut builder, GoldilocksField(36)),
            Variable::constant(&mut builder, GoldilocksField(221)),
            Variable::constant(&mut builder, GoldilocksField(181)),
            Variable::constant(&mut builder, GoldilocksField(53)),
            Variable::constant(&mut builder, GoldilocksField(160)),
            Variable::constant(&mut builder, GoldilocksField(58)),
            Variable::constant(&mut builder, GoldilocksField(167)),
            Variable::constant(&mut builder, GoldilocksField(131)),
            Variable::constant(&mut builder, GoldilocksField(216)),
            Variable::constant(&mut builder, GoldilocksField(183)),
            Variable::constant(&mut builder, GoldilocksField(83)),
            Variable::constant(&mut builder, GoldilocksField(232)),
            Variable::constant(&mut builder, GoldilocksField(151)),
            Variable::constant(&mut builder, GoldilocksField(87)),
            Variable::constant(&mut builder, GoldilocksField(46)),
            Variable::constant(&mut builder, GoldilocksField(54)),
            Variable::constant(&mut builder, GoldilocksField(128)),
            Variable::constant(&mut builder, GoldilocksField(123)),
            Variable::constant(&mut builder, GoldilocksField(231)),
            Variable::constant(&mut builder, GoldilocksField(212)),
            Variable::constant(&mut builder, GoldilocksField(130)),
            Variable::constant(&mut builder, GoldilocksField(19)),
            Variable::constant(&mut builder, GoldilocksField(28)),
            Variable::constant(&mut builder, GoldilocksField(96)),
            Variable::constant(&mut builder, GoldilocksField(108)),
            Variable::constant(&mut builder, GoldilocksField(111)),
            Variable::constant(&mut builder, GoldilocksField(137)),
            Variable::constant(&mut builder, GoldilocksField(154)),
            Variable::constant(&mut builder, GoldilocksField(40)),
            Variable::constant(&mut builder, GoldilocksField(184)),
            Variable::constant(&mut builder, GoldilocksField(74)),
            Variable::constant(&mut builder, GoldilocksField(69)),
            Variable::constant(&mut builder, GoldilocksField(100)),
            Variable::constant(&mut builder, GoldilocksField(64)),
            Variable::constant(&mut builder, GoldilocksField(177)),
            Variable::constant(&mut builder, GoldilocksField(98)),
            Variable::constant(&mut builder, GoldilocksField(248)),
            Variable::constant(&mut builder, GoldilocksField(32)),
            Variable::constant(&mut builder, GoldilocksField(12)),
            Variable::constant(&mut builder, GoldilocksField(97)),
            Variable::constant(&mut builder, GoldilocksField(49)),
            Variable::constant(&mut builder, GoldilocksField(187)),
            Variable::constant(&mut builder, GoldilocksField(39)),
            Variable::constant(&mut builder, GoldilocksField(159)),
            Variable::constant(&mut builder, GoldilocksField(168)),
            Variable::constant(&mut builder, GoldilocksField(247)),
            Variable::constant(&mut builder, GoldilocksField(29)),
            Variable::constant(&mut builder, GoldilocksField(246)),
            Variable::constant(&mut builder, GoldilocksField(209)),
            Variable::constant(&mut builder, GoldilocksField(110)),
            Variable::constant(&mut builder, GoldilocksField(77)),
            Variable::constant(&mut builder, GoldilocksField(73)),
            Variable::constant(&mut builder, GoldilocksField(20)),
            Variable::constant(&mut builder, GoldilocksField(23)),
            Variable::constant(&mut builder, GoldilocksField(174)),
            Variable::constant(&mut builder, GoldilocksField(143)),
            Variable::constant(&mut builder, GoldilocksField(93)),
            Variable::constant(&mut builder, GoldilocksField(92)),
            Variable::constant(&mut builder, GoldilocksField(162)),
            Variable::constant(&mut builder, GoldilocksField(48)),
            Variable::constant(&mut builder, GoldilocksField(134)),
            Variable::constant(&mut builder, GoldilocksField(119)),
            Variable::constant(&mut builder, GoldilocksField(213)),
            Variable::constant(&mut builder, GoldilocksField(139)),
            Variable::constant(&mut builder, GoldilocksField(234)),
            Variable::constant(&mut builder, GoldilocksField(205)),
            Variable::constant(&mut builder, GoldilocksField(91)),
            Variable::constant(&mut builder, GoldilocksField(113)),
            Variable::constant(&mut builder, GoldilocksField(204)),
            Variable::constant(&mut builder, GoldilocksField(121)),
            Variable::constant(&mut builder, GoldilocksField(57)),
            Variable::constant(&mut builder, GoldilocksField(4)),
            Variable::constant(&mut builder, GoldilocksField(41)),
            Variable::constant(&mut builder, GoldilocksField(180)),
            Variable::constant(&mut builder, GoldilocksField(144)),
            Variable::constant(&mut builder, GoldilocksField(76)),
            Variable::constant(&mut builder, GoldilocksField(107)),
            Variable::constant(&mut builder, GoldilocksField(59)),
            Variable::constant(&mut builder, GoldilocksField(176)),
            Variable::constant(&mut builder, GoldilocksField(43)),
            Variable::constant(&mut builder, GoldilocksField(11)),
            Variable::constant(&mut builder, GoldilocksField(127)),
            Variable::constant(&mut builder, GoldilocksField(34)),
            Variable::constant(&mut builder, GoldilocksField(38)),
            Variable::constant(&mut builder, GoldilocksField(164)),
            Variable::constant(&mut builder, GoldilocksField(9)),
            Variable::constant(&mut builder, GoldilocksField(141)),
            Variable::constant(&mut builder, GoldilocksField(78)),
            Variable::constant(&mut builder, GoldilocksField(245)),
            Variable::constant(&mut builder, GoldilocksField(175)),
            Variable::constant(&mut builder, GoldilocksField(145)),
            Variable::constant(&mut builder, GoldilocksField(112)),
            Variable::constant(&mut builder, GoldilocksField(129)),
            Variable::constant(&mut builder, GoldilocksField(109)),
            Variable::constant(&mut builder, GoldilocksField(18)),
            Variable::constant(&mut builder, GoldilocksField(250)),
            Variable::constant(&mut builder, GoldilocksField(85)),
            Variable::constant(&mut builder, GoldilocksField(16)),
            Variable::constant(&mut builder, GoldilocksField(124)),
            Variable::constant(&mut builder, GoldilocksField(182)),
            Variable::constant(&mut builder, GoldilocksField(242)),
            Variable::constant(&mut builder, GoldilocksField(158)),
            Variable::constant(&mut builder, GoldilocksField(84)),
            Variable::constant(&mut builder, GoldilocksField(219)),
            Variable::constant(&mut builder, GoldilocksField(13)),
            Variable::constant(&mut builder, GoldilocksField(207)),
            Variable::constant(&mut builder, GoldilocksField(186)),
            Variable::constant(&mut builder, GoldilocksField(82)),
            Variable::constant(&mut builder, GoldilocksField(157)),
            Variable::constant(&mut builder, GoldilocksField(132)),
            Variable::constant(&mut builder, GoldilocksField(225)),
            Variable::constant(&mut builder, GoldilocksField(236)),
            Variable::constant(&mut builder, GoldilocksField(45)),
            Variable::constant(&mut builder, GoldilocksField(185)),
            Variable::constant(&mut builder, GoldilocksField(228)),
            Variable::constant(&mut builder, GoldilocksField(161)),
            Variable::constant(&mut builder, GoldilocksField(169)),
            Variable::constant(&mut builder, GoldilocksField(106)),
            Variable::constant(&mut builder, GoldilocksField(25)),
            Variable::constant(&mut builder, GoldilocksField(155)),
            Variable::constant(&mut builder, GoldilocksField(251)),
            Variable::constant(&mut builder, GoldilocksField(254)),
            Variable::constant(&mut builder, GoldilocksField(223)),
        ];
        let hash_to_curve_res = hash_to_curve(&mut builder, &msg);

        // Define your circuit.
        let mut res_output: Vec<GoldilocksField> = Vec::new();
        for i in 0..hash_to_curve_res.len() {
            for j in 0..hash_to_curve_res[i].len() {
                for k in 0..LIMBS {
                    builder.write(Variable(hash_to_curve_res[i][j].limbs[k].target));
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
        for i in 0..hash_to_curve_res.len() {
            for _ in 0..hash_to_curve_res[i].len() {
                for _ in 0..LIMBS {
                    res_output.push(output.read::<Variable>())
                }
            }
        }

        let mut biguint_res: Vec<BigUint> = Vec::new();
        for i in 0..4 {
            biguint_res.push(BigUint::new(
                res_output[(i * 12)..(i * 12) + 12]
                    .iter()
                    .map(|f| (f.0 % GoldilocksField::ORDER) as u32)
                    .collect_vec(),
            ));
        }

        let expected_biguint_targets = vec![
                    BigUint::from_str("263045359310876400672266134102422923142170786488971463144260837991310793708919904974750654695723771449817953534932").unwrap(), 
                    BigUint::from_str("705085714867347375204839501082774976133427291820427587421388912165231801117635419620551803041968063138400265133663").unwrap(), 
                    BigUint::from_str("3303090097836311338780356548102458653001297014651905027382930947462021925827856111160646227318455068671696298599273").unwrap(), 
                    BigUint::from_str("2746000687320669913100540339419677393886381993350402195421358168305846473266968075760380449244083602094512053359154").unwrap()
                ];

        for i in 0..4 {
            assert_eq!(biguint_res[i], expected_biguint_targets[i]);
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

        // Define your circuit.
        let mut res_output: Vec<GoldilocksField> = Vec::new();
        for i in 0..iso_map_r.len() {
            for j in 0..iso_map_r[i].len() {
                for k in 0..LIMBS {
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
            for _ in 0..iso_map_r[i].len() {
                for _ in 0..LIMBS {
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
            assert_eq!(biguint_res[i], expected_biguint_targets[i]);
        }
    }

    #[test]
    fn test_clear_cofactor() {
        let mut builder = DefaultBuilder::new();
        let x = [builder.api.constant_biguint(&BigUint::from_str("474682481268733588266168000983897038833463740369371343293271315606510847229825856506681723856424762498931536081381").unwrap()), builder.api.constant_biguint(&BigUint::from_str("1366297191634768530389324840135632614622170346303255080801396974208665528754948924260000453159829725659141010218083").unwrap())];
        let new_point = map_to_curve_simple_swu_9mod16(&mut builder, &x);
        let iso_map_r = isogeny_map(&mut builder, &new_point);
        let clear_cofactor = clear_cofactor_g2(&mut builder, &iso_map_r);

        // Define your circuit.
        let mut res_output: Vec<GoldilocksField> = Vec::new();
        for i in 0..clear_cofactor.len() {
            for j in 0..clear_cofactor[i].len() {
                for k in 0..LIMBS {
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
            for _ in 0..clear_cofactor[i].len() {
                for _ in 0..LIMBS {
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
            BigUint::from_str("1333544920615259474714661371327518954416732544068349411293275363187401395459274109080234631197310432595159920946891").unwrap(),
            BigUint::from_str("3534898797471258007317464582418403172692698020727006028871480408936368621561281829419543100267410234420500056142147").unwrap(),
            BigUint::from_str("3937050676002649672972543005965063406357492217339476444626945930452046333693534966501454095684077919472794301839550").unwrap(),
            BigUint::from_str("2505850057307810573716759564908795048162371887702901637040931176762748544745723014444120460791457110594458168503549").unwrap()
        ];

        for i in 0..4 {
            assert_eq!(biguint_res[i], expected_biguint_targets[i]);
        }
    }
}
