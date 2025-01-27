/*
 * SPDX-FileCopyrightText: 2025 Daisuke Nagao
 *
 * SPDX-License-Identifier: MIT
 */

use clap::Parser;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short = 'o', long = "output")]
    filename: Option<String>,

    #[arg(long, default_value_t = false)]
    overwrite: bool,

    #[arg(long = "prefix", default_value = "UUID")]
    prefix: String,

    #[arg(long = "suffix", default_value = None)]
    suffix: Option<String>,
}

fn generate_guard(prefix: String, suffix: Option<String>) -> String {
    let uuid = uuid7::uuid7().to_string().replace('-', "_").to_uppercase();
    let mut guard = vec![prefix, uuid];
    if let Some(suffix) = suffix {
        guard.push(suffix);
    }

    let guard = guard.join("_");

    let ifndef = format!("#ifndef {}", guard);
    let define = format!("#define {}", guard);
    let endif = format!("#endif /* {} */", guard);

    let mut text = vec![ifndef, define];

    text.push(endif);
    text.push("".to_string());

    text.join("\n")
}

fn main() {
    let args = Args::parse();

    let guard = generate_guard(args.prefix, args.suffix);

    if let Some(file_path) = &args.filename {
        if !args.overwrite && fs::metadata(file_path).is_ok() {
            eprintln!(
                "Error: File '{}' already exists. Use --overwrite to overwrite.",
                file_path
            );
            std::process::exit(1);
        }

        match OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(args.overwrite)
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
