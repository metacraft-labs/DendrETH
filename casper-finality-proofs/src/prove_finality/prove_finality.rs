use plonky2x::{backend::circuit::Circuit, prelude::{PlonkParameters, CircuitBuilder, BoolVariable, U64Variable}};
use crate::weigh_justification_and_finalization::epoch_processing::{get_previous_epoch, get_current_epoch};


#[derive(Debug, Clone)]
pub struct ProveFinality;

impl Circuit for ProveFinality {
    fn define<L: PlonkParameters<D>, const D: usize>(builder: &mut CircuitBuilder<L, D>) {
        let zero_bit = builder.constant::<BoolVariable>(false);
        let _one_bit = builder.constant::<BoolVariable>(true);
        let _32 = builder.constant::<U64Variable>(32);
        let test_bit_mask = vec![zero_bit; 16_000_000];
        let current_epoch = get_current_epoch(builder, _32);
        let previous_epoch = get_previous_epoch(builder, current_epoch);
    }
}
