// use std::mem::MaybeUninit;
//
// use circuit::{Circuit, CircuitInput, SetWitness, TargetPrimitiveType};
// use circuit_derive::TargetPrimitive;
// use circuits::{
//     common_targets::ValidatorTarget,
//     validators_commitment_mapper::first_level::ValidatorsCommitmentMapperFirstLevel,
// };
// use criterion::{
//     black_box, criterion_group, criterion_main, measurement::WallTime, BenchmarkGroup, Criterion,
// };
// use plonky2::{field::goldilocks_field::GoldilocksField, iop::witness::PartialWitness};
//
// fn bench_and_return<T>(
//     group: &mut BenchmarkGroup<WallTime>,
//     bench_name: &str,
//     closure: impl Fn() -> T,
// ) -> T {
//     // NOTE: pseudo-warmup: this is also needed since I could't find a way to return
//     // the result of the benchmarked function, so this execution is not recorded
//     let result: T = closure();
//     group.bench_function(bench_name, |b| b.iter(|| closure()));
//     result
// }
//
// fn circuit_benchmarks(c: &mut Criterion) {
//     let mut group = c.benchmark_group("validators commitment mapper first level circuit");
//
//     let (target, data) = bench_and_return(
//         &mut group,
//         "validators commitment mapper first level circuit",
//         || ValidatorsCommitmentMapperFirstLevel::build(&()),
//     );
//
//     let validator = std::mem::MaybeUninit::uninit();
//     let input = CircuitInput::<ValidatorsCommitmentMapperFirstLevel> {
//         validator: unsafe { validator.assume_init() },
//         is_real: false,
//     };
//
//     let mut pw = PartialWitness::<GoldilocksField>::new();
//     target.set_witness(&mut pw, &input);
//
//     group.finish();
// }
//
// criterion_group! {
//     name = benches;
//     config = Criterion::default().sample_size(10);
//     targets = circuit_benchmarks
// }
// criterion_main!(benches);
