#[macro_export]
macro_rules! make_uint32_n {
    ($a:ident, $b:ty, $c:expr) => {
        #[derive(Debug, Clone, Copy)]
        pub struct $a {
            pub limbs: [U32Target; $c],
        }

        impl crate::TargetPrimitive for $a {
            type Primitive = $b;
        }

        impl<F: RichField> SetWitness<F> for $a {
            type Input = <$a as TargetPrimitive>::Primitive;

            fn set_witness(&self, witness: &mut PartialWitness<F>, input: &Self::Input) {
                for (index, &limb) in self.limbs.iter().enumerate() {
                    let value: u32 = ((input >> (32 * index))
                        & Self::Input::from(0xffffffff as u32))
                    .try_into()
                    .unwrap();
                    witness.set_u32_target(limb, value as u32);
                }
            }
        }

        impl AddVirtualTarget for $a {
            fn add_virtual_target<F: RichField + Extendable<D>, const D: usize>(
                builder: &mut CircuitBuilder<F, D>,
            ) -> Self {
                let targets = builder.add_virtual_u32_targets($c);
                assert_limbs_are_valid(builder, &targets);
                Self {
                    limbs: targets.try_into().unwrap(),
                }
            }
        }

        impl PublicInputsReadable for $a {
            fn from_elements<F: RichField>(elements: &[F]) -> Self::Primitive {
                assert_eq!(elements.len(), Self::get_size());
                elements
                    .iter()
                    .rev()
                    .fold(Self::Primitive::from(0u64), |acc, limb| {
                        (acc << 32) + Self::Primitive::from(limb.to_canonical_u64())
                    })
            }
        }

        impl PublicInputsTargetReadable for $a {
            fn get_size() -> usize {
                $c
            }

            fn from_targets(targets: &[Target]) -> Self {
                assert_eq!(targets.len(), Self::get_size());
                Self {
                    limbs: targets
                        .iter()
                        .map(|&target| U32Target(target))
                        .collect_vec()
                        .try_into()
                        .unwrap(),
                }
            }
        }

        impl ToTargets for $a {
            fn to_targets(&self) -> Vec<Target> {
                self.limbs.iter().map(|limb| limb.0).collect_vec()
            }
        }

        impl<F: RichField + Extendable<D>, const D: usize> Zero<F, D> for $a {
            fn zero(builder: &mut CircuitBuilder<F, D>) -> Self {
                Self {
                    limbs: [U32Target(builder.zero()); $c],
                }
            }
        }

        impl<F: RichField + Extendable<D>, const D: usize> One<F, D> for $a {
            fn one(builder: &mut CircuitBuilder<F, D>) -> Self {
                let zero = U32Target(builder.zero());
                let one = U32Target(builder.one());
                let mut limbs = [zero; $c];
                limbs[0] = one;
                Self { limbs }
            }
        }

        impl<F: RichField + Extendable<D>, const D: usize> Add<F, D> for $a {
            type Output = Self;

            fn add(self, rhs: $a, builder: &mut CircuitBuilder<F, D>) -> Self::Output {
                let self_targets = self
                    .limbs
                    .iter()
                    .map(|x| U32Target::from(*x))
                    .collect::<Vec<_>>();
                let rhs_targets = rhs
                    .limbs
                    .iter()
                    .map(|x| U32Target::from(*x))
                    .collect::<Vec<_>>();

                let self_biguint = BigUintTarget {
                    limbs: self_targets,
                };
                let rhs_biguint = BigUintTarget { limbs: rhs_targets };
                let sum_biguint = builder.add_biguint(&self_biguint, &rhs_biguint);

                let mut limbs: [U32Target; $c] = Self::zero(builder).limbs;
                for i in 0..$c {
                    limbs[i] = sum_biguint.limbs[i].into();
                }

                Self { limbs }
            }
        }
    };
}
