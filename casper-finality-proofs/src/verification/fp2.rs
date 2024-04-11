use num_bigint::ToBigUint;
use plonky2::{
    field::extension::Extendable,
    hash::hash_types::RichField,
    iop::{
        generator::{GeneratedValues, SimpleGenerator},
        target::{BoolTarget, Target},
        witness::PartitionWitness,
    },
    plonk::circuit_data::CommonCircuitData,
    util::serialization::{Buffer, IoResult},
};
use plonky2x::{
    backend::circuit::PlonkParameters,
    frontend::{
        builder::CircuitBuilder,
        uint::num::biguint::{CircuitBuilderBiguint, GeneratedValuesBigUint, WitnessBigUint},
        vars::BoolVariable,
    },
};

use crate::verification::fp::{
    add_fp, fp_is_equal, fp_is_zero, mul_fp, negate_fp, range_check_fp, sub_fp, FpTarget, N,
};
use crate::verification::native::{Fp, Fp2};

use super::fp::{deserialize, serialize};

const E: usize = 2;
pub type Fp2Target = [FpTarget; E];

pub fn is_zero<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    input: &Fp2Target,
) -> BoolVariable {
    let zero1 = fp_is_zero(builder, &input[0]);
    let zero2 = fp_is_zero(builder, &input[1]);
    builder.and(zero1, zero2)
}

pub fn is_equal<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    a: &Fp2Target,
    b: &Fp2Target,
) -> BoolVariable {
    BoolVariable::from(a.iter().zip(b.iter()).fold(
        builder.api.constant_bool(true),
        |acc, (a_f, b_f)| {
            let is_equal = fp_is_equal(builder, a_f, b_f);
            builder
                .api
                .and(acc, BoolTarget::new_unsafe(is_equal.variable.0))
        },
    ))
}

pub fn range_check_fp2<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    input: &Fp2Target,
) {
    range_check_fp(builder, &input[0]);
    range_check_fp(builder, &input[1]);
}

pub fn sgn0_fp2<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    input: &Fp2Target,
) -> BoolVariable {
    let two = builder.api.constant_biguint(&2u32.into());
    let sign0 = builder.api.rem_biguint(&input[0], &two);
    let sign0_bool = BoolTarget::new_unsafe(sign0.limbs[0].target);
    let zero0 = fp_is_zero(builder, &input[0]);
    let sign1 = builder.api.rem_biguint(&input[1], &two);
    let sign1_bool = BoolTarget::new_unsafe(sign1.limbs[0].target);
    let zero_and_sign1 = builder.and(zero0, BoolVariable::from(sign1_bool));
    builder.or(BoolVariable::from(sign0_bool), zero_and_sign1)
}

pub fn add_fp2<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    a: &Fp2Target,
    b: &Fp2Target,
) -> Fp2Target {
    let mut res = vec![];
    for i in 0..E {
        res.push(add_fp(builder, &a[i], &b[i]));
    }
    res.try_into().unwrap()
}

pub fn negate_fp2<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    input: &Fp2Target,
) -> Fp2Target {
    let mut res = vec![];
    for i in 0..E {
        res.push(negate_fp(builder, &input[i]));
    }
    res.try_into().unwrap()
}

pub fn sub_fp2<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    a: &Fp2Target,
    b: &Fp2Target,
) -> Fp2Target {
    let minus_b = negate_fp2(builder, b);
    add_fp2(builder, a, &minus_b)
}

pub fn mul_fp2<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    a: &Fp2Target,
    b: &Fp2Target,
) -> Fp2Target {
    let t1 = mul_fp(builder, &a[0], &b[0]);
    let t2 = mul_fp(builder, &a[1], &b[1]);
    let t1t2 = add_fp(builder, &t1, &t2);

    let c0c1 = add_fp(builder, &a[0], &a[1]);
    let r0r1 = add_fp(builder, &b[0], &b[1]);
    let c0c1r0r1 = mul_fp(builder, &c0c1, &r0r1);

    let mut res = vec![];
    res.push(sub_fp(builder, &t1, &t2));
    res.push(sub_fp(builder, &c0c1r0r1, &t1t2));
    res.try_into().unwrap()
}

pub fn inv_fp2<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    input: &Fp2Target,
) -> Fp2Target {
    let one = builder.api.constant_biguint(&1u32.to_biguint().unwrap());
    let zero = builder.api.constant_biguint(&0u32.to_biguint().unwrap());
    let inv_c0 = builder.api.add_virtual_biguint_target_unsafe(N);
    let inv_c1 = builder.api.add_virtual_biguint_target_unsafe(N);
    let input_inv = [inv_c0, inv_c1];
    builder.api.add_simple_generator(Fp2InverseGenerator {
        input: input.clone(),
        input_inv: input_inv.clone(),
    });
    range_check_fp2(builder, &input_inv);
    let mul = mul_fp2(builder, input, &input_inv);
    builder.api.connect_biguint(&mul[0], &one);
    builder.api.connect_biguint(&mul[1], &zero);
    input_inv
}

pub fn div_fp2<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    a: &Fp2Target,
    b: &Fp2Target,
) -> Fp2Target {
    let b_inv = inv_fp2(builder, b);
    mul_fp2(builder, a, &b_inv)
}

pub fn frobenius_map<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    inp: &Fp2Target,
    pow: usize,
) -> Fp2Target {
    if pow % 2 == 0 {
        inp.clone()
    } else {
        [inp[0].clone(), negate_fp(builder, &inp[1])]
    }
}

#[derive(Debug, Default)]
pub struct Fp2InverseGenerator {
    input: Fp2Target,
    input_inv: Fp2Target,
}

impl<F: RichField + Extendable<D>, const D: usize> SimpleGenerator<F, D> for Fp2InverseGenerator {
    fn id(&self) -> String {
        "Fp2InverseGenerator".to_string()
    }

    fn dependencies(&self) -> Vec<Target> {
        self.input
            .iter()
            .flat_map(|f| f.limbs.iter().map(|l| l.target))
            .collect::<Vec<Target>>()
    }

    fn run_once(&self, witness: &PartitionWitness<F>, out_buffer: &mut GeneratedValues<F>) {
        let c0 = witness.get_biguint_target(self.input[0].clone());
        let c1 = witness.get_biguint_target(self.input[1].clone());
        let input_fp2 = Fp2([Fp::get_fp_from_biguint(c0), Fp::get_fp_from_biguint(c1)]);
        let inverse = input_fp2.invert();
        out_buffer.set_biguint_target(&self.input_inv[0], &inverse.0[0].to_biguint());
        out_buffer.set_biguint_target(&self.input_inv[1], &inverse.0[1].to_biguint());
    }

    fn serialize(&self, dst: &mut Vec<u8>, _common_data: &CommonCircuitData<F, D>) -> IoResult<()> {
        serialize(self.input[0].clone(), dst)?;
        serialize(self.input[1].clone(), dst)?;
        serialize(self.input_inv[0].clone(), dst)?;
        serialize(self.input_inv[1].clone(), dst)
    }

    fn deserialize(src: &mut Buffer, _common_data: &CommonCircuitData<F, D>) -> IoResult<Self>
    where
        Self: Sized,
    {
        let c0 = deserialize(src)?;
        let c1 = deserialize(src)?;
        let inv_c0 = deserialize(src)?;
        let inv_c1 = deserialize(src)?;
        Ok(Self {
            input: [c0, c1],
            input_inv: [inv_c0, inv_c1],
        })
    }
}

// #[cfg(test)]
// mod tests {
//     use plonky2::{
//         iop::witness::PartialWitness,
//         plonk::{
//             circuit_builder::CircuitBuilder,
//             circuit_data::CircuitConfig,
//             config::{GenericConfig, PoseidonGoldilocksConfig},
//         },
//     };
//     use plonky2_crypto::biguint::{CircuitBuilderBiguint, WitnessBigUint};

//     use crate::{
//         fp_plonky2::N,
//         native::{Fp, Fp2},
//     };

//     use super::{div_fp2, sub_fp2};

//     const D: usize = 2;
//     type C = PoseidonGoldilocksConfig;
//     type F = <C as GenericConfig<D>>::F;

//     #[test]
//     fn test_subtraction_circuit() {
//         env_logger::init();
//         let config = CircuitConfig::standard_recursion_config();

//         let mut builder = CircuitBuilder::<F, D>::new(config);
//         let c0 = builder.add_virtual_biguint_target(N);
//         let c1 = builder.add_virtual_biguint_target(N);
//         let r0 = builder.add_virtual_biguint_target(N);
//         let r1 = builder.add_virtual_biguint_target(N);
//         let a = [c0, c1];
//         let b = [r0, r1];
//         let res = sub_fp2(&mut builder, &a, &b);

//         builder.print_gate_counts(0);
//         let data = builder.build::<C>();
//         let mut pw = PartialWitness::<F>::new();

//         let c0_fp = Fp([
//             1115400077, 734036635, 2658976793, 3446373348, 3797461211, 2799729988, 1086715089,
//             1766116042, 3720719530, 4214563288, 2211874409, 287824937,
//         ]);
//         let c1_fp = Fp([
//             4070035387, 3598430679, 2371795623, 2598602036, 314293284, 3104159902, 3828298491,
//             1770882328, 1026148559, 2003704675, 804131021, 382850433,
//         ]);
//         let r0_fp = Fp([
//             3944640261, 440162500, 3767697757, 767512216, 3185360355, 1355179671, 2310853452,
//             2890628660, 2539693039, 3306767406, 473197245, 198293246,
//         ]);
//         let r1_fp = Fp([
//             920955909, 775806582, 2117093864, 286632291, 2248224021, 4208799968, 2272086148,
//             4009382258, 291945614, 2017047933, 1541154483, 220533456,
//         ]);
//         let a_fp2 = Fp2([c0_fp, c1_fp]);
//         let b_fp2 = Fp2([r0_fp, r1_fp]);
//         let res_fp2 = a_fp2 - b_fp2;
//         pw.set_biguint_target(&a[0], &a_fp2.0[0].to_biguint());
//         pw.set_biguint_target(&a[1], &a_fp2.0[1].to_biguint());
//         pw.set_biguint_target(&b[0], &b_fp2.0[0].to_biguint());
//         pw.set_biguint_target(&b[1], &b_fp2.0[1].to_biguint());
//         pw.set_biguint_target(&res[0], &res_fp2.0[0].to_biguint());
//         pw.set_biguint_target(&res[1], &res_fp2.0[1].to_biguint());

//         let proof = data.prove(pw).unwrap();
//         data.verify(proof).unwrap();
//     }

//     #[test]
//     fn test_division_circuit() {
//         env_logger::init();
//         let config = CircuitConfig::standard_recursion_config();

//         let mut builder = CircuitBuilder::<F, D>::new(config);
//         let c0 = builder.add_virtual_biguint_target(N);
//         let c1 = builder.add_virtual_biguint_target(N);
//         let r0 = builder.add_virtual_biguint_target(N);
//         let r1 = builder.add_virtual_biguint_target(N);
//         let a = [c0, c1];
//         let b = [r0, r1];
//         let res = div_fp2(&mut builder, &a, &b);
//         builder.print_gate_counts(0);

//         let data = builder.build::<C>();
//         let mut pw = PartialWitness::<F>::new();

//         let c0_fp = Fp([
//             1115400077, 734036635, 2658976793, 3446373348, 3797461211, 2799729988, 1086715089,
//             1766116042, 3720719530, 4214563288, 2211874409, 287824937,
//         ]);
//         let c1_fp = Fp([
//             4070035387, 3598430679, 2371795623, 2598602036, 314293284, 3104159902, 3828298491,
//             1770882328, 1026148559, 2003704675, 804131021, 382850433,
//         ]);
//         let r0_fp = Fp([
//             3944640261, 440162500, 3767697757, 767512216, 3185360355, 1355179671, 2310853452,
//             2890628660, 2539693039, 3306767406, 473197245, 198293246,
//         ]);
//         let r1_fp = Fp([
//             920955909, 775806582, 2117093864, 286632291, 2248224021, 4208799968, 2272086148,
//             4009382258, 291945614, 2017047933, 1541154483, 220533456,
//         ]);
//         let a_fp2 = Fp2([c0_fp, c1_fp]);
//         let b_fp2 = Fp2([r0_fp, r1_fp]);
//         let res_fp2 = a_fp2 / b_fp2;
//         pw.set_biguint_target(&a[0], &a_fp2.0[0].to_biguint());
//         pw.set_biguint_target(&a[1], &a_fp2.0[1].to_biguint());
//         pw.set_biguint_target(&b[0], &b_fp2.0[0].to_biguint());
//         pw.set_biguint_target(&b[1], &b_fp2.0[1].to_biguint());
//         pw.set_biguint_target(&res[0], &res_fp2.0[0].to_biguint());
//         pw.set_biguint_target(&res[1], &res_fp2.0[1].to_biguint());

//         let proof = data.prove(pw).unwrap();
//         data.verify(proof).unwrap();
//     }
// }
