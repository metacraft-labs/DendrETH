use num_bigint::ToBigUint;
use plonky2::{
    field::extension::Extendable,
    hash::hash_types::RichField,
    iop::{
        generator::{GeneratedValues, SimpleGenerator},
        target::Target,
        witness::PartitionWitness,
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
        vars::BoolVariable,
    },
};

use crate::verification::utils::native_bls::{mod_inverse, modulus};

pub const N: usize = 12;
pub type FpTarget = BigUintTarget;

pub fn serialize(fp: FpTarget, dst: &mut Vec<u8>) -> plonky2::util::serialization::IoResult<()> {
    dst.write_target_vec(&fp.limbs.iter().map(|bt| bt.target).collect::<Vec<Target>>())
}

pub fn deserialize(src: &mut Buffer) -> IoResult<FpTarget> {
    let target_limbs = src.read_target_vec()?;
    let limbs: Vec<U32Target> = target_limbs
        .into_iter()
        .map(|f| U32Target::from_target_unsafe(f))
        .collect();
    Ok(FpTarget { limbs })
}

pub fn fp_is_zero<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    input: &FpTarget,
) -> BoolVariable {
    let zero = builder.api.zero_biguint();
    BoolVariable::from(builder.api.cmp_biguint(input, &zero))
}

pub fn fp_is_equal<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    a: &FpTarget,
    b: &FpTarget,
) -> BoolVariable {
    BoolVariable::from(a.limbs.iter().zip(b.limbs.iter()).fold(
        builder.api.constant_bool(true),
        |acc, (a_l, b_l)| {
            let is_equal = builder.api.is_equal(a_l.target, b_l.target);
            builder.api.and(acc, is_equal)
        },
    ))
}

pub fn range_check_fp<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    input: &FpTarget,
) {
    let p = builder.api.constant_biguint(&modulus());
    let check = builder.api.cmp_biguint(&p, &input);
    builder.api.assert_zero(check.target);
}

pub fn add_fp<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    a: &FpTarget,
    b: &FpTarget,
) -> FpTarget {
    let zero = builder.api.zero();
    let p = builder.api.constant_biguint(&modulus());
    let res = builder.api.add_biguint(a, b);
    let cmp = builder.api.cmp_biguint(&p, &res);
    let sub_limbs = (0..12)
        .into_iter()
        .map(|i| U32Target::from_target_unsafe(builder.api.select(cmp, p.limbs[i].target, zero)))
        .collect::<Vec<U32Target>>();
    let sub = BigUintTarget { limbs: sub_limbs };
    builder.api.sub_biguint(&res, &sub)
}

pub fn negate_fp<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    input: &FpTarget,
) -> FpTarget {
    let p = builder.api.constant_biguint(&modulus());
    builder.api.sub_biguint(&p, input)
}

pub fn sub_fp<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    a: &FpTarget,
    b: &FpTarget,
) -> FpTarget {
    let minus_b = negate_fp(builder, b);
    add_fp(builder, a, &minus_b)
}

pub fn mul_fp<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    a: &FpTarget,
    b: &FpTarget,
) -> FpTarget {
    let p = builder.api.constant_biguint(&modulus());
    let res = builder.api.mul_biguint(a, b);
    builder.api.rem_biguint(&res, &p)
}

pub fn inv_fp<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    input: &FpTarget,
) -> FpTarget {
    let one = builder.api.constant_biguint(&1u32.to_biguint().unwrap());
    let input_inv = builder.api.add_virtual_biguint_target_unsafe(N);
    builder.api.add_simple_generator(FpInverseGenerator {
        input: input.clone(),
        input_inv: input_inv.clone(),
    });
    range_check_fp(builder, &input_inv);
    let mul = mul_fp(builder, input, &input_inv);
    builder.api.connect_biguint(&mul, &one);
    input_inv
}

pub fn div_fp<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    a: &FpTarget,
    b: &FpTarget,
) -> FpTarget {
    let b_inv = inv_fp(builder, b);
    mul_fp(builder, a, &b_inv)
}

#[derive(Debug, Default)]
pub struct FpInverseGenerator {
    input: BigUintTarget,
    input_inv: BigUintTarget,
}

impl<F: RichField + Extendable<D>, const D: usize> SimpleGenerator<F, D> for FpInverseGenerator {
    fn id(&self) -> String {
        "FpInverseGenerator".to_string()
    }

    fn dependencies(&self) -> Vec<Target> {
        self.input
            .limbs
            .iter()
            .map(|l| l.target)
            .collect::<Vec<Target>>()
    }

    fn run_once(&self, witness: &PartitionWitness<F>, out_buffer: &mut GeneratedValues<F>) {
        let input = witness.get_biguint_target(self.input.clone());
        let inverse = mod_inverse(input, modulus());
        out_buffer.set_biguint_target(&self.input_inv, &inverse);
    }

    fn serialize(&self, dst: &mut Vec<u8>, _common_data: &CommonCircuitData<F, D>) -> IoResult<()> {
        serialize(self.input.clone(), dst)?;
        serialize(self.input_inv.clone(), dst)
    }

    fn deserialize(src: &mut Buffer, _common_data: &CommonCircuitData<F, D>) -> IoResult<Self>
    where
        Self: Sized,
    {
        let input = deserialize(src)?;
        let input_inv = deserialize(src)?;
        Ok(Self { input, input_inv })
    }
}
// #[cfg(test)]
// mod tests {
//     use std::str::FromStr;

//     use num_bigint::BigUint;
//     use plonky2::{
//         iop::witness::PartialWitness,
//         plonk::{
//             circuit_data::CircuitConfig,
//             config::{GenericConfig, PoseidonGoldilocksConfig},
//         },
//     };
//     use plonky2x::{
//         backend::circuit::{Circuit, CircuitBuild, DefaultParameters, PlonkParameters},
//         frontend::{
//             builder::CircuitBuilder,
//             uint::num::biguint::{BigUintTarget, CircuitBuilderBiguint, WitnessBigUint},
//         },
//     };

//     use crate::verification::native::Fp;

//     use super::{div_fp, range_check_fp, sub_fp, N};

//     const D: usize = 2;
//     type C = PoseidonGoldilocksConfig;
//     type F = <C as GenericConfig<D>>::F;

//     #[test]
//     fn test_subtraction_circuit() {
//         type L = DefaultParameters;
//         const D: usize = 2;
//         let mut builder: CircuitBuilder<DefaultParameters, 2> =
//             CircuitBuilder::<DefaultParameters, 2>::new();
//         let circuit = builder.build();
//         let mut input = circuit.input();

//         let a = builder.add_virtual_biguint_target(N);
//         let b = builder.add_virtual_biguint_target(N);
//         let res = sub_fp(&mut builder, &a, &b);

//         let mut pw = PartialWitness::<F>::new();

//         let a_fp = Fp::get_fp_from_biguint(BigUint::from_str(
//             "1216495682195235861952885506871698490232894470117269383940381148575524314493849307811227440691167647909822763414941"
//         ).unwrap());
//         let b_fp = Fp::get_fp_from_biguint(BigUint::from_str(
//             "2153848155426317245700560287567131132765685008362732985860101000686875894603366983854567186180519945327668975076337"
//         ).unwrap());
//         let res_fp = a_fp - b_fp;
//         input.write::<BigUintTarget>(a);
//         pw.set_biguint_target(&a, &a_fp.to_biguint());
//         pw.set_biguint_target(&b, &b_fp.to_biguint());
//         pw.set_biguint_target(&res, &res_fp.to_biguint());

//         let proof = data.prove(pw).unwrap();
//         data.verify(proof).unwrap();
//     }

//     #[test]
//     fn test_division_circuit() {
//         let config = CircuitConfig::standard_recursion_config();

//         let mut builder = CircuitBuilder::<F, D>::new(config);
//         let a = builder.add_virtual_biguint_target(N);
//         let b = builder.add_virtual_biguint_target(N);
//         let res = div_fp(&mut builder, &a, &b);

//         let data = builder.build::<C>();
//         let mut pw = PartialWitness::<F>::new();

//         let a_fp = Fp::get_fp_from_biguint(BigUint::from_str(
//             "2153848155426317245700560287567131132765685008362732985860101000686875894603366983854567186180519945327668975076337"
//         ).unwrap());
//         let b_fp = Fp::get_fp_from_biguint(BigUint::from_str(
//             "1216495682195235861952885506871698490232894470117269383940381148575524314493849307811227440691167647909822763414941"
//         ).unwrap());
//         let res_fp = a_fp / b_fp;
//         pw.set_biguint_target(&a, &a_fp.to_biguint());
//         pw.set_biguint_target(&b, &b_fp.to_biguint());
//         pw.set_biguint_target(&res, &res_fp.to_biguint());

//         let proof = data.prove(pw).unwrap();
//         data.verify(proof).unwrap();
//     }

//     #[test]
//     fn test_range_check_fp() {
//         env_logger::init();
//         let config = CircuitConfig::standard_recursion_config();

//         let mut builder = CircuitBuilder::<F, D>::new(config);
//         let input = builder.add_virtual_biguint_target(N);

//         range_check_fp(&mut builder, &input);

//         builder.print_gate_counts(0);
//         let data = builder.build::<C>();
//         let mut pw = PartialWitness::<F>::new();

//         pw.set_biguint_target(&input, &BigUint::from_str("234").unwrap());
//         let proof = data.prove(pw).unwrap();
//         data.verify(proof).unwrap();
//     }
// }
