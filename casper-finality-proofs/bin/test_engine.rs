use casper_finality_proofs::test_engine::utils::test_engine::{build_function_map, init_tests};
use colored::Colorize;
use crossbeam::thread;
use std::panic;

fn main() {
    // Set a custom panic hook
    panic::set_hook(Box::new(|info| {
        println!(
            "Error: {}",
            info.payload().downcast_ref::<String>().unwrap()
        );
    }));

    let tests = init_tests();

    let function_map = build_function_map(tests);
    let mut failed_tests: Vec<String> = Vec::new();

    for (name, _) in function_map.iter() {
        println!("\nRunning circuit: {}", format!("{:?}", name).blue().bold());
        let folder_path = &function_map.get(&name).unwrap().folder_path;
        let files = std::fs::read_dir(folder_path).unwrap();

        for file in files {
            let file = file.unwrap();
            let path = file.path().to_str().unwrap().to_owned();
            let file_name = file.file_name().to_str().unwrap().to_owned();
            let mut colored_file_name = String::from(file_name.clone()).green();

            let r = thread::scope(|s| {
                s.spawn(|_| {
                    return (function_map.get(name).unwrap().wrapper)(path);
                });
            });

            match r {
                Ok(_) => {}
                Err(e) => {
                    let mut error_str = String::from("Circuit failure");
                    if let Some(e) = e.downcast_ref::<&'static str>() {
                        error_str = format!("Error: {}", e);
                    } else if let Some(e) = e.downcast_ref::<String>() {
                        error_str = format!("Error: {}", e);
                    }
                    colored_file_name = String::from(file_name.clone()).on_red();
                    failed_tests.push(format!(
                        "{}: {}",
                        String::from(file_name).yellow(),
                        error_str
                    ));
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
