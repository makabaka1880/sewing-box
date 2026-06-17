// Created by Sean L. on Jun. 17.
// Last Updated by Sean L. on Jun. 17.
//
// wasm-lambda
// src/main.rs
//
// Makabaka1880, 2026. All rights reserved.

use std::io::{Read, stdin};

use wasm_lambda::{interpreter::KrivMachine, parser::parse};

fn main() {
    let mut store = Vec::new();
    stdin()
        .read_to_end(&mut store)
        .expect("failed to read stdin");
    let src = std::str::from_utf8(&store).expect("input is not valid UTF-8");

    let sexp = sexp::parse(src).unwrap_or_else(|e| {
        eprintln!("S-expression parse error: {:?}", e);
        std::process::exit(1);
    });
    let term = parse(sexp).unwrap_or_else(|e| {
        eprintln!("Parse error: {}", e);
        std::process::exit(1);
    });

    let mut machine = KrivMachine::new(term);
    match machine.evaluate() {
        Ok((term, _env)) => println!("=> {:?}", term),
        Err(e) => eprintln!("Evaluation error: {}", e),
    }
}
