/*
 * SPDX-FileCopyrightText: 2025 Daisuke Nagao
 *
 * SPDX-License-Identifier: MIT
 */

 use std::env;
use std::fs;
use std::fs::OpenOptions;
use std::io::{self, Write};

fn generate_guard() -> String {
    let uuid = uuid7::uuid7();
    format!(
        "#ifndef UUID_{}
#define UUID_{}


#endif /* UUID_{} */",
        uuid.to_string().replace('-', "_"),
        uuid.to_string().replace('-', "_"),
        uuid.to_string().replace('-', "_")
    )
}

fn main() {
    let args: Vec<String> = env::args().collect();

    // Parse arguments
    let mut filename: Option<&str> = None;
    let mut overwrite = false;

    let mut args_iter = args.iter();
    while let Some(arg) = args_iter.next() {
        match arg.as_str() {
            "--output" | "-o" => {
                if let Some(file) = args_iter.next() {
                    filename = Some(file);
                } else {
                    eprintln!("Error: Missing value for {}", arg);
                    std::process::exit(1);
                }
            }
            "--overwrite" => {
                overwrite = true;
            }
            _ => {}
        }
    }

    let guard = generate_guard();

    if let Some(file) = filename {
        let file_path = file;
        if !overwrite && fs::metadata(file_path).is_ok() {
            eprintln!(
                "Error: File '{}' already exists. Use --overwrite to overwrite.",
                file_path
            );
            std::process::exit(1);
        }

        match OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(overwrite)
            .open(file_path)
        {
            Ok(mut file) => {
                if let Err(e) = file.write_all(guard.as_bytes()) {
                    eprintln!("Error writing to file '{}': {}", file_path, e);
                    std::process::exit(1);
                }
                println!("Guard written to '{}'.", file_path);
            }
            Err(e) => {
                eprintln!("Error creating file '{}': {}", file_path, e);
                std::process::exit(1);
            }
        }
    } else {
        println!("{}", guard);
    }
}
