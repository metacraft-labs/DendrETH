use casper_finality_proofs::test_engine::utils::test_engine::{build_function_map, init_tests};
use colored::Colorize;

fn main() {
    let tests = init_tests();

    let function_map = build_function_map(tests);
    let mut failed_tests: Vec<String> = Vec::new();

    for (name, _) in &function_map {
        println!("\nRunning circuit: {}", format!("{:?}", name).blue().bold());
        let folder_path = &function_map.get(name).unwrap().folder_path;
        let files = std::fs::read_dir(folder_path).unwrap();

        for file in files {
            let file = file.unwrap();
            let path = file.path().to_str().unwrap().to_owned();
            let file_name = file.file_name().to_str().unwrap().to_owned();
            let mut colored_file_name = String::from(file_name.clone()).green();

            let res = (function_map.get(name).unwrap().wrapper)(path);
            match res {
                Err(e) => {
                    colored_file_name = String::from(file_name.clone()).on_red();
                    failed_tests.push(format!("{}: {:?}", String::from(file_name).yellow(), e));
                }
                _ => {}
            };
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
