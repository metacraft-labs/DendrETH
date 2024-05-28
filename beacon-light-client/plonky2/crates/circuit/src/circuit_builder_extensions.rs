use plonky2::{
    field::extension::Extendable, hash::hash_types::RichField, iop::target::BoolTarget,
    plonk::circuit_builder::CircuitBuilder,
};

use crate::{PublicInputsTargetReadable, ToTargets};

pub trait CircuitBuilderExtensions {
    fn select_target<T>(&mut self, selector: BoolTarget, a: &T, b: &T) -> T
    where
        T: ToTargets + PublicInputsTargetReadable;

    fn imply(&mut self, p: BoolTarget, q: BoolTarget) -> BoolTarget;
}

impl<F: RichField + Extendable<D>, const D: usize> CircuitBuilderExtensions
    for CircuitBuilder<F, D>
{
    fn select_target<T>(&mut self, selector: BoolTarget, a: &T, b: &T) -> T
    where
        T: ToTargets + PublicInputsTargetReadable,
    {
        let a_targets = a.to_targets();
        let b_targets = b.to_targets();

        let pairs = a_targets.iter().zip(b_targets.iter());

        let targets = pairs.fold(vec![], |mut acc, (&a_target, &b_target)| {
            acc.push(self._if(selector, a_target, b_target));
            acc
        });

        T::from_targets(&targets)
    }

    /// p -> q, could also be written as !p || q
    fn imply(&mut self, p: BoolTarget, q: BoolTarget) -> BoolTarget {
        let not_p = self.not(p);
        self.or(not_p, q)
    }
}
