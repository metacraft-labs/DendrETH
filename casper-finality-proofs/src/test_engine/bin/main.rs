use casper_finality_proofs::test_engine::utils::{
    setup::init_tests,
    test_engine::{build_function_map, handle_error},
};
use colored::Colorize;
use crossbeam::thread;
use std::{env, panic};

static FAIL_EXT: &str = "_fail.json";

fn main() {
    // Prevent the program from stopping its execution on panic.
    panic::set_hook(Box::new(|_| {}));

    let args: Vec<String> = env::args().skip(1).collect();
    let mut tests = init_tests();

    if args.len() > 0 {
        let test = tests.iter().find(|test| test.name.to_string() == args[0]);
        match test {
            Some(t) => {
                tests = vec![t.clone()];
            }
            None => {
                println!(
                    "{}",
                    format!("Test {} not found.", args[0].to_string().blue())
                        .bold()
                        .red()
                );
                return;
            }
        }
    }

    let function_map = build_function_map(tests);
    let mut failed_tests: Vec<String> = Vec::new();

    for (name, _) in function_map.iter() {
        let circuit_name = format!("{:?}", name).blue().bold();
        println!("\nRunning circuit: {}", circuit_name);
        let folder_path = &function_map.get(&name).unwrap().folder_path;
        let files = std::fs::read_dir(folder_path).unwrap();

        for file in files {
            let file = file.unwrap();
            let path = file.path().to_str().unwrap().to_owned();
            let file_name = file.file_name().to_str().unwrap().to_owned();
            let mut colored_file_name = String::from(file_name.clone()).green();

            let r = thread::scope(|s| {
                let join_handle = s.spawn(|_| {
                    return (function_map.get(name).unwrap().wrapper)(path);
                });

                let res = join_handle.join();
                return res;
            });

            match r.unwrap() {
                // Thread finished without panic.
                Ok(r) => {
                    // Assertion failed inside wrapper.
                    if let Err(e) = r {
                        handle_error(
                            Box::new(e),
                            &mut colored_file_name,
                            &file_name,
                            &circuit_name,
                            &mut failed_tests,
                        );
                    } else if file_name.contains(FAIL_EXT) {
                        handle_error(
                            Box::new("Test is supposed to fail but it passed."),
                            &mut colored_file_name,
                            &file_name,
                            &circuit_name,
                            &mut failed_tests,
                        );
                    }
                }
                // Thread panicked due to circuit failure when called inside wrapper.
                Err(e) => {
                    if !file_name.contains(FAIL_EXT) {
                        handle_error(
                            e,
                            &mut colored_file_name,
                            &file_name,
                            &circuit_name,
                            &mut failed_tests,
                        );
                    }
                }
            }

            println!("-> {}", colored_file_name);
        }
    }

    if failed_tests.len() > 0 {
        println!("\n{}", "Failed tests:".red().bold());
        for test in failed_tests {
            println!("-> {}", test);
        }
    } else {
        println!("\n{}", "All tests passed!".green().bold());
    }
}
