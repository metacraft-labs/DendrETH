use casper_finality_proofs::test_engine::utils::{
    setup::init_tests,
    test_engine::{build_function_map, handle_error},
};
use clap::Parser;
use colored::Colorize;
use crossbeam::thread;
use std::panic;
use walkdir::WalkDir;

static FAIL_EXT: &str = "_fail.";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value_t = String::from(""))]
    circuit: String,

    #[arg(short, long, default_value_t = String::from(""))]
    path: String,

    #[arg(short, long, default_value_t = false)]
    r#ref: bool,
}

fn main() {
    // Prevent the program from stopping its execution on panic.
    panic::set_hook(Box::new(|_| {}));

    let args = Args::parse();
    let mut tests = init_tests();

    if !args.circuit.is_empty() {
        let test = tests
            .iter()
            .find(|test| test.name.to_string() == args.circuit);
        match test {
            Some(t) => {
                tests = vec![t.clone()];
            }
            None => {
                println!(
                    "{}",
                    format!("Test {} not found.", args.circuit.blue())
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
        let folder_path = if args.path.is_empty() {
            &function_map.get(&name).unwrap().folder_path
        } else {
            &args.path
        };

        for e in WalkDir::new(folder_path).into_iter().filter_map(|e| e.ok()) {
            if e.metadata().unwrap().is_file() {
                let path = e.path().display().to_string();
                let file_name = if args.r#ref {
                    path.clone()
                } else {
                    e.file_name().to_str().unwrap().to_owned()
                };

                let r = thread::scope(|s| {
                    let join_handle = s.spawn(|_| {
                        return (function_map.get(name).unwrap().wrapper)(path, !args.r#ref);
                    });

                    let res = join_handle.join();
                    return res;
                });

                let handle_success = |res: String| {
                    let colored_file_name = String::from(file_name.clone()).green();
                    if args.r#ref {
                        println!("{} {} {}", "[OK]".green().bold(), colored_file_name, res);
                    } else {
                        println!("-> {}", colored_file_name);
                    }
                };

                match r.unwrap() {
                    // Thread finished without panic.
                    Ok(r) => {
                        // Assertion failed inside wrapper.
                        if let Err(e) = r {
                            handle_error(
                                Box::new(e),
                                args.r#ref,
                                &file_name,
                                &circuit_name,
                                &mut failed_tests,
                            );
                        } else if file_name.contains(FAIL_EXT) {
                            handle_error(
                                Box::new("Test is supposed to fail but it passed."),
                                args.r#ref,
                                &file_name,
                                &circuit_name,
                                &mut failed_tests,
                            );
                        } else {
                            handle_success(r.unwrap());
                        }
                    }
                    // Thread panicked due to circuit failure when called inside wrapper.
                    Err(e) => {
                        if !file_name.contains(FAIL_EXT) {
                            handle_error(
                                e,
                                args.r#ref,
                                &file_name,
                                &circuit_name,
                                &mut failed_tests,
                            );
                        } else {
                            handle_success("".to_string());
                        }
                    }
                }
            }
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
