use std::{
    collections::{HashMap, HashSet},
    rc::Rc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

pub struct BenchmarkConfig {
    pub sample_size: u64,
}

#[derive(Debug)]
pub struct FlattenedBenchmarkResults(pub Vec<BenchmarkResult>);

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BenchmarkResult {
    pub name: String,
    pub parent: Option<String>,
    pub start_time: Duration,
    pub end_time: u64,
    pub sub_benchmarks: Vec<BenchmarkResult>,
}

impl FlattenedBenchmarkResults {
    pub fn unflatten(&self) -> BenchmarkResult {
        let mut benchmarks_lookup: HashMap<String, BenchmarkResult> = HashMap::new();

        for benchmark in self.0.iter().cloned() {
            benchmarks_lookup.insert(benchmark.name.clone(), benchmark);
        }

        let asd: *mut BenchmarkResult = None;

        for benchmark in self.0.iter() {
            if let Some(ref parent_name) = benchmark.parent {
                let parent_benchmark = benchmarks_lookup.get_mut(parent_name).unwrap();
                parent_benchmark.sub_benchmarks.push(benchmark.clone());
            }
        }

        // println!("benchmarks_lookup: {:?}", benchmarks_lookup);

        benchmarks_lookup
            .iter()
            .find(|(_, benchmark)| benchmark.parent.is_none())
            .unwrap()
            .1
            .clone()
    }
}

pub struct Benchmark {
    pub name: String,
    pub function: fn(&mut Benchmark),
    pub scopes: Vec<BenchmarkScope>,
    pub flattened_microbenchmark_results: Vec<FlattenedBenchmarkResults>,
}

impl Benchmark {
    pub fn new(name: &str, function: fn(&mut Benchmark)) -> Self {
        Self {
            name: name.to_owned(),
            function,
            scopes: Vec::new(),
            flattened_microbenchmark_results: Vec::new(),
        }
    }

    pub fn run(&mut self, config: BenchmarkConfig) {
        assert!(self.scopes.is_empty());

        for _ in 0..config.sample_size {
            self.begin();
            (self.function)(self);
            self.end();
        }
    }

    pub fn begin(&mut self) {
        let base_scope = BenchmarkScope::new_base(&self.name);
        self.scopes.push(base_scope.clone());

        self.flattened_microbenchmark_results
            .push(FlattenedBenchmarkResults(Vec::<BenchmarkResult>::new()));
    }

    pub fn end(&mut self) {
        let base_scope = BenchmarkScope::new_base(&self.name);

        self.flattened_microbenchmark_results
            .last_mut()
            .unwrap()
            .0
            .push(BenchmarkResult {
                name: base_scope.name.clone(),
                parent: base_scope.parent_name.clone(),
                start_time: base_scope.start_time,
                end_time: base_scope.end_time,
                sub_benchmarks: Vec::new(),
            });

        self.scopes.pop();
    }

    pub fn begin_scope(&mut self, name: impl Into<String>) {
        let new_scope = BenchmarkScope::new_child(name, &self.scopes.last().unwrap().name);
        self.scopes.push(new_scope);
    }

    pub fn end_scope(&mut self, name: &str) {
        if self.scopes.len() == 0 {
            panic!("Ending benchmark scope but no scopes are pushed");
        }

        if self.scopes.last().unwrap().name != name {
            panic!(
                "Ending benchmark scope {name} but {} is on top",
                self.scopes.last().unwrap().name
            );
        }

        let top_scope = self.scopes.last().unwrap();

        self.flattened_microbenchmark_results
            .last_mut()
            .unwrap()
            .0
            .push(BenchmarkResult {
                name: top_scope.name.clone(),
                parent: top_scope.parent_name.clone(),
                start_time: top_scope.start_time,
                end_time: top_scope.end_time,
                sub_benchmarks: Vec::new(),
            });

        self.scopes.pop();
    }
}

#[derive(Clone)]
pub struct BenchmarkScope {
    pub parent_name: Option<String>,
    pub name: String,
    pub start_time: Duration,
    pub end_time: u64,
}

impl BenchmarkScope {
    pub fn new_base(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            parent_name: None,
            start_time: SystemTime::now().duration_since(UNIX_EPOCH).unwrap(),
            end_time: 0,
        }
    }

    pub fn new_child(name: impl Into<String>, parent: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            parent_name: Some(parent.into()),
            start_time: SystemTime::now().duration_since(UNIX_EPOCH).unwrap(),
            end_time: 0,
        }
    }
}
