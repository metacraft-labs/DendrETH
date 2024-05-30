use plonky2::{
    field::extension::Extendable,
    hash::hash_types::RichField,
    iop::target::{BoolTarget, Target},
    plonk::circuit_builder::CircuitBuilder,
};

use crate::{PublicInputsTargetReadable, ToTargets};

pub trait CircuitBuilderExtensions {
    fn select_target<T>(&mut self, selector: BoolTarget, a: &T, b: &T) -> T
    where
        T: ToTargets + PublicInputsTargetReadable;

    fn imply(&mut self, p: BoolTarget, q: BoolTarget) -> BoolTarget;

    fn assert_implication(&mut self, p: BoolTarget, q: BoolTarget);

    fn assert_true(&mut self, b: BoolTarget);

    fn assert_false(&mut self, b: BoolTarget);

    fn zero_init<T: PublicInputsTargetReadable>(&mut self) -> T;

    fn targets_are_equal<T: ToTargets>(&mut self, a: &T, b: &T) -> BoolTarget;

    fn assert_targets_are_equal<T: ToTargets>(&mut self, a: &T, b: &T);
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

    fn assert_implication(&mut self, p: BoolTarget, q: BoolTarget) {
        let implication = self.imply(p, q);
        self.assert_true(implication);
    }

    fn assert_true(&mut self, b: BoolTarget) {
        let _true = self._true();
        self.connect(b.target, _true.target);
    }

    fn assert_false(&mut self, b: BoolTarget) {
        let _false = self._false();
        self.connect(b.target, _false.target);
    }

    fn zero_init<T: PublicInputsTargetReadable>(&mut self) -> T {
        let zeroes = vec![self.zero(); T::get_size()];
        T::from_targets(&zeroes)
    }

    fn targets_are_equal<T: ToTargets>(&mut self, a: &T, b: &T) -> BoolTarget {
        let a_targets = a.to_targets();
        let b_targets = b.to_targets();
        arrays_are_equal(self, &a_targets, &b_targets)
    }

    fn assert_targets_are_equal<T: ToTargets>(&mut self, a: &T, b: &T) {
        let targets_are_equal = self.targets_are_equal(a, b);
        self.assert_true(targets_are_equal);
    }
}

fn arrays_are_equal<F: RichField + Extendable<D>, const D: usize>(
    builder: &mut CircuitBuilder<F, D>,
    first: &[Target],
    second: &[Target],
) -> BoolTarget {
    assert!(first.len() == second.len());

    let mut result = builder._true();
    for idx in 0..first.len() {
        let is_equal = builder.is_equal(first[idx], second[idx]);
        result = builder.and(result, is_equal);
    }
    result
}
