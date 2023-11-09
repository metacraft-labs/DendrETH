use std::marker::PhantomData;

use curta::math::prelude::PrimeField64;
use plonky2::{
    iop::{
        generator::{GeneratedValues, SimpleGenerator},
        witness::PartitionWitness,
    },
    plonk::circuit_data::CommonCircuitData,
    util::serialization::IoResult,
};
use plonky2x::prelude::{
    CircuitBuilder, CircuitVariable, Field, PlonkParameters, Target, Variable, WitnessWrite,
};

use super::plonky2x_extensions::assert_is_true;

#[derive(Debug, Clone)]
struct SetNthBitGenerator<L: PlonkParameters<D>, const D: usize> {
    bitset: Variable,
    n: Variable,
    pub result: Variable,
    pub difference: Variable,
    _phantom: PhantomData<L>,
}

impl<L: PlonkParameters<D>, const D: usize> SetNthBitGenerator<L, D> {
    pub fn new(builder: &mut CircuitBuilder<L, D>, bitset: Variable, n: Variable) -> Self {
        Self {
            bitset,
            n,
            result: Variable::init(builder),
            difference: Variable::init(builder),
            _phantom: PhantomData,
        }
    }

    pub fn id() -> String {
        "SetNthBitGenerator".to_string()
    }
}

impl<L: PlonkParameters<D>, const D: usize> SimpleGenerator<L::Field, D>
    for SetNthBitGenerator<L, D>
{
    fn id(&self) -> String {
        Self::id()
    }

    fn dependencies(&self) -> Vec<Target> {
        vec![self.bitset.0, self.n.0]
    }

    #[allow(unused_variables)]
    fn serialize(
        &self,
        dst: &mut Vec<u8>,
        _common_data: &CommonCircuitData<L::Field, D>,
    ) -> IoResult<()> {
        todo!()
    }

    #[allow(unused_variables)]
    fn deserialize(
        src: &mut plonky2::util::serialization::Buffer,
        _common_data: &CommonCircuitData<L::Field, D>,
    ) -> IoResult<Self>
    where
        Self: Sized,
    {
        todo!()
    }

    fn run_once(
        &self,
        witness: &PartitionWitness<L::Field>,
        out_buffer: &mut GeneratedValues<L::Field>,
    ) {
        let bitset = self.bitset.get(witness).as_canonical_u64();
        let n = self.n.get(witness).as_canonical_u64();

        let bitset_with_set_nth_bit = bitset | (1 << n);
        let difference = bitset_with_set_nth_bit - bitset;

        out_buffer.set_target(
            self.result.0,
            <L as PlonkParameters<D>>::Field::from_canonical_u64(bitset_with_set_nth_bit),
        );

        out_buffer.set_target(
            self.difference.0,
            <L as PlonkParameters<D>>::Field::from_canonical_u64(difference),
        );
    }
}

pub fn variable_set_nth_bit<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    bitset: Variable,
    n: Variable,
    powers_of_two: &[Variable],
) -> Variable {
    let generator = SetNthBitGenerator::new(builder, bitset, n);
    builder.add_simple_generator(generator.clone());

    let nth_power_of_two = builder.select_array(&powers_of_two, n);
    let difference_is_nth_power_of_two_pred =
        builder.is_equal(generator.difference, nth_power_of_two);
    let difference_is_zero_pred = builder.is_zero(generator.difference);
    let generated_value_is_valid_pred =
        builder.or(difference_is_nth_power_of_two_pred, difference_is_zero_pred);
    assert_is_true(builder, generated_value_is_valid_pred);

    generator.result
}
