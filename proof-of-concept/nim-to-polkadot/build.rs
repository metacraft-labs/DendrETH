#![allow(unused)]
use std::io::{self, Write};
use std::process::Command;
use symlink::{remove_symlink_file, symlink_file};
use std::env;
use std::env::VarError;
use std::env::var_os;


fn main() -> Result<(), VarError> {
    symlink_file("../panicoverride.nim", "panicoverride.nim");

    let output = Command::new("nim")
        .arg("c")
        .arg("-c")
        .arg("--noMain")
        .arg("--noLinking")
        .arg("--nimcache:nimcache")
        .arg("lib.nim")
        .output()
        .expect("Failed to invoke nim compiler");
    if !output.status.success() {
        let msg = String::from_utf8_lossy(output.stderr.as_slice());
        let _ = writeln!(io::stderr(), "\nerror occurred: {}\n", msg);
        std::process::exit(1);
    }

    cc::Build::new()
        // .compiler("clang")
        .compiler("gcc")
        .no_default_flags(true)
        .include("..")
        .warnings(false)
        .target("wasm32-unknown-unknown-wasm")
        .flag("-m32")
        .flag("-nostdinc")
        .flag("-fno-builtin")
        .flag("-fno-exceptions")
        .flag("-fno-threadsafe-statics")
        .flag("-fvisibility=hidden")
        .flag("-flto")
        .flag("-std=c99")
        .file("nimcache/@mlib.nim.c")
        .file("nimcache/stdlib_system.nim.c")
        .shared_flag(true)
        .static_flag(true)
        .out_dir("target/ink/release/deps")
        .compile("nim");

        println!("cargo:rustc-link-lib=nim");

        // switch("clang.options.linker", "--target=wasm32-unknown-unknown-wasm -nostdlib -Wl,--no-entry,--allow-undefined,--export-dynamic,--gc-sections,--strip-all")

    remove_symlink_file("panicoverride.nim");

    Ok(())
}
