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
    BoolVariable, CircuitBuilder, CircuitVariable, Field, PlonkParameters, Target, Variable,
    WitnessWrite,
};

use super::plonky2x_extensions::assert_is_false;

#[derive(Debug, Clone)]
struct VariableIntDivGenerator<L: PlonkParameters<D>, const D: usize> {
    numerator: Variable,
    denominator: Variable,
    pub quotient: Variable,
    pub remainder: Variable,
    _phantom: PhantomData<L>,
}

impl<L: PlonkParameters<D>, const D: usize> VariableIntDivGenerator<L, D> {
    pub fn new(
        builder: &mut CircuitBuilder<L, D>,
        numerator: Variable,
        denominator: Variable,
    ) -> Self {
        Self {
            numerator,
            denominator,
            quotient: Variable::init(builder),
            remainder: Variable::init(builder),
            _phantom: PhantomData,
        }
    }

    pub fn id() -> String {
        "VariableIntDivGenerator".to_string()
    }
}

impl<L: PlonkParameters<D>, const D: usize> SimpleGenerator<L::Field, D>
    for VariableIntDivGenerator<L, D>
{
    fn id(&self) -> String {
        Self::id()
    }

    fn dependencies(&self) -> Vec<Target> {
        vec![self.numerator.0, self.denominator.0]
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
        let numerator = self.numerator.get(witness).as_canonical_u64();
        let denominator = self.denominator.get(witness).as_canonical_u64();

        let quotient = numerator / denominator;
        let remainder = numerator % denominator;

        out_buffer.set_target(
            self.quotient.0,
            <L as PlonkParameters<D>>::Field::from_canonical_u64(quotient),
        );

        out_buffer.set_target(
            self.remainder.0,
            <L as PlonkParameters<D>>::Field::from_canonical_u64(remainder),
        );
    }
}

pub fn variable_int_div_rem<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    numerator: Variable,
    denominator: Variable,
) -> (Variable, Variable) {
    let generator = VariableIntDivGenerator::new(builder, numerator, denominator);
    builder.add_simple_generator(generator.clone());

    let quotient_times_denominator = builder.mul(generator.quotient, denominator);
    let computed_numerator = builder.add(quotient_times_denominator, generator.remainder);
    builder.assert_is_equal(numerator, computed_numerator);

    let denominator_is_zero_pred = builder.is_zero(denominator);
    assert_is_false(builder, denominator_is_zero_pred);

    (generator.quotient, generator.remainder)
}

pub fn variable_int_div<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    numerator: Variable,
    denominator: Variable,
) -> Variable {
    let (quotient, _) = variable_int_div_rem(builder, numerator, denominator);
    quotient
}

pub fn variable_int_rem<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    numerator: Variable,
    denominator: Variable,
) -> Variable {
    let (_, remainder) = variable_int_div_rem(builder, numerator, denominator);
    remainder
}

pub fn is_power_of_two<L: PlonkParameters<D>, const D: usize>(
    builder: &mut CircuitBuilder<L, D>,
    variable: Variable,
) -> BoolVariable {
    let two = builder.constant::<Variable>(<L as PlonkParameters<D>>::Field::from_canonical_u64(2));

    let mut result = builder._false();
    let mut power_of_two = builder.one();
    for _ in 0..64 {
        let pred = builder.is_equal(variable, power_of_two);
        result = builder.or(result, pred);
        power_of_two = builder.mul(power_of_two, two);
    }

    result
}
