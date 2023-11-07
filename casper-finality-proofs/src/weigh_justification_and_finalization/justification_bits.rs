use plonky2x::prelude::RichField;
use plonky2x::{
    frontend::vars::SSZVariable,
    prelude::{
        ArrayVariable, BoolVariable, ByteVariable, Bytes32Variable, CircuitBuilder,
        CircuitVariable, PlonkParameters, Variable,
    },
};

use crate::utils::plonky2x_extensions::shift_right;

#[derive(Debug, Clone, CircuitVariable)]
#[value_name(JustificationBitsValue)]
pub struct JustificationBitsVariable {
    pub bits: ArrayVariable<BoolVariable, 4>,
}

impl JustificationBitsVariable {
    pub fn test_range<L: PlonkParameters<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<L, D>,
        lower_bound: usize,
        upper_bound_non_inclusive: usize,
    ) -> BoolVariable {
        let mut result = builder._true();
        for i in lower_bound..upper_bound_non_inclusive {
            result = builder.and(result, self.bits[i]);
        }
        result
    }

    pub fn shift_right<L: PlonkParameters<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<L, D>,
    ) -> JustificationBitsVariable {
        JustificationBitsVariable {
            bits: ArrayVariable::new(shift_right(builder, self.bits.as_slice(), 1)),
        }
    }

    pub fn assign_nth_bit(&self, n: usize, value: BoolVariable) -> JustificationBitsVariable {
        let mut new_bits = self.bits.as_vec();
        new_bits[n] = value;
        JustificationBitsVariable {
            bits: ArrayVariable::new(new_bits),
        }
    }
}

impl SSZVariable for JustificationBitsVariable {
    fn hash_tree_root<L: PlonkParameters<D>, const D: usize>(
        &self,
        builder: &mut CircuitBuilder<L, D>,
    ) -> Bytes32Variable {
        let zero_byte = builder.constant::<ByteVariable>(0);
        let zero_bit = builder.constant::<BoolVariable>(false);

        let first_byte = ByteVariable([
            zero_bit,
            zero_bit,
            zero_bit,
            zero_bit,
            self.bits[3],
            self.bits[2],
            self.bits[1],
            self.bits[0],
        ]);

        let mut justification_bits_vec = vec![first_byte];
        justification_bits_vec.extend(vec![zero_byte; 31]);
        let justification_bits_fixed_size: [ByteVariable; 32] =
            justification_bits_vec.try_into().unwrap();

        let justification_bits_leaf = Bytes32Variable::from(justification_bits_fixed_size);
        justification_bits_leaf
    }
}
