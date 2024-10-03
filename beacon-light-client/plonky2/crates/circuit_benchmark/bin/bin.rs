use std::{thread::sleep, time::Duration};

use circuit_benchmark::benchmark::{Benchmark, BenchmarkConfig};

fn benchmark_function(benchmark: &mut Benchmark) {
    benchmark.begin_scope("inner");
    let mut a = 1 + 2;

    benchmark.begin_scope("bench loop");
    for i in 0..100 {
        a += i;
    }
    benchmark.end_scope("bench loop");

    sleep(Duration::from_secs(1));
    println!("a = {a}: {}", benchmark.scopes.len());

    benchmark.end_scope("inner");
}

pub fn main() {
    let mut benchmark = Benchmark::new("benchmark function", benchmark_function);
    benchmark.run(BenchmarkConfig { sample_size: 2 });

    // println!("{:?}", benchmark.flattened_microbenchmark_results);
    println!(
        "{:?}",
        benchmark.flattened_microbenchmark_results[0].unflatten()
    );
}
