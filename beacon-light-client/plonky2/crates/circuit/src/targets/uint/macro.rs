#[macro_export]
macro_rules! make_uint32_n {
    ($a:ident, $b:ty, $c:expr) => {
        /// An integer type encoded as little-endian u32 limbs.
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
                for (index, limb) in self.limbs.into_iter().enumerate() {
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
            #[allow(arithmetic_overflow)]
            fn from_elements<F: RichField>(elements: &[F]) -> Self::Primitive {
                assert_eq!(elements.len(), Self::get_size());
                elements
                    .iter()
                    .rev()
                    .fold(Self::Primitive::from(0u32), |acc, limb| {
                        (acc << 32) + Self::Primitive::from(limb.to_canonical_u64() as u32)
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
                let self_biguint = BigUintTarget {
                    limbs: self.limbs.to_vec(),
                };
                let rhs_biguint = BigUintTarget {
                    limbs: rhs.limbs.to_vec(),
                };
                let sum_biguint = builder.add_biguint(&self_biguint, &rhs_biguint);

                let mut limbs: [U32Target; $c] = Self::zero(builder).limbs;
                for i in 0..$c {
                    limbs[i] = sum_biguint.limbs[i].into();
                }

                Self { limbs }
            }
        }
        impl<F: RichField + Extendable<D>, const D: usize> Div<F, D> for $a {
            type Output = Self;

            fn div(self, rhs: $a, builder: &mut CircuitBuilder<F, D>) -> Self::Output {
                let self_biguint = BigUintTarget {
                    limbs: self.limbs.to_vec(),
                };
                let rhs_biguint = BigUintTarget {
                    limbs: rhs.limbs.to_vec(),
                };
                let quotient_biguint = builder.div_biguint(&self_biguint, &rhs_biguint);

                let mut limbs: [U32Target; $c] = Self::zero(builder).limbs;
                for i in 0..quotient_biguint.num_limbs() {
                    limbs[i] = quotient_biguint.limbs[i].into();
                }

                Self { limbs }
            }
        }
        impl<F: RichField + Extendable<D>, const D: usize> Mul<F, D> for $a {
            type Output = Self;

            fn mul(self, rhs: $a, builder: &mut CircuitBuilder<F, D>) -> Self::Output {
                let self_biguint = BigUintTarget {
                    limbs: self.limbs.to_vec(),
                };
                let rhs_biguint = BigUintTarget {
                    limbs: rhs.limbs.to_vec(),
                };
                let product_biguint = builder.mul_biguint(&self_biguint, &rhs_biguint);

                let mut limbs: [U32Target; $c] = Self::zero(builder).limbs;
                for i in 0..$c {
                    limbs[i] = product_biguint.limbs[i].into();
                }

                Self { limbs }
            }
        }
        impl<F: RichField + Extendable<D>, const D: usize> Rem<F, D> for $a {
            type Output = Self;

            fn rem(self, rhs: $a, builder: &mut CircuitBuilder<F, D>) -> Self::Output {
                let self_biguint = BigUintTarget {
                    limbs: self.limbs.to_vec(),
                };
                let rhs_biguint = BigUintTarget {
                    limbs: rhs.limbs.to_vec(),
                };
                let rem_biguint = builder.rem_biguint(&self_biguint, &rhs_biguint);

                let mut limbs: [U32Target; $c] = Self::zero(builder).limbs;
                for i in 0..$c {
                    limbs[i] = rem_biguint.limbs[i].into();
                }

                Self { limbs }
            }
        }
        impl<F: RichField + Extendable<D>, const D: usize> LessThanOrEqual<F, D> for $a {
            #[must_use]
            fn lte(self, rhs: Self, builder: &mut CircuitBuilder<F, D>) -> BoolTarget {
                let self_biguint = BigUintTarget {
                    limbs: self.limbs.to_vec(),
                };
                let rhs_biguint = BigUintTarget {
                    limbs: rhs.limbs.to_vec(),
                };
                builder.cmp_biguint(&self_biguint, &rhs_biguint)
            }
        }
    };
}
