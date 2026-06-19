// Created by Sean L. on Jun. 18.
// Last Updated by Sean L. on Jun. 18.
//
// brainfuck-wasm
// src/parser.rs
//
// Makabaka1880, 2026. All rights reserved.

pub enum Tokens {
    IncCell,
    DecCell,
    IncPtr,
    DecPtr,
    BrLeft,
    PrRight,
    Print,
    Read,
}

pub fn parse(src: &str) -> Vec<Tokens> {
    src.chars()
        .filter(|c| matches!(c, '+' | '-' | '<' | '>' | '[' | ']' | '.' | ','))
        .map(|c| match c {
            '+' => Tokens::IncCell,
            '-' => Tokens::DecCell,
            '>' => Tokens::IncPtr,
            '<' => Tokens::DecPtr,
            '[' => Tokens::BrLeft,
            ']' => Tokens::PrRight,
            '.' => Tokens::Print,
            ',' => Tokens::Read,
            _ => unreachable!(),
        })
        .collect()
}
