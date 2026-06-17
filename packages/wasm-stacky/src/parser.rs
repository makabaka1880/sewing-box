// Created by Sean L. on Jun. 17.
// Last Updated by Sean L. on Jun. 17.
//
// sewing-box
// packages/wasm-stacky/src/parser.rs
//
// Makabaka1880, 2026. All rights reserved.

use serde::Serialize;
use sexp::{Atom, Sexp};

#[derive(Clone, Debug, Serialize)]
pub enum Stmt {
    Push(String),
    Plant(String, u32),
}

pub fn parse(src: Sexp) -> Result<Stmt, String> {
    let list = match src {
        Sexp::List(sexps) => sexps,
        _ => return Err("expected a list".into()),
    };
    match list.as_slice() {
        [op, args @ ..] => {
            let op = match op {
                Sexp::Atom(Atom::S(s)) => s.as_ref(),
                _ => return Err("expected an atom as operator".into()),
            };
            match (op, args) {
                ("push", [Sexp::Atom(Atom::S(val))]) => Ok(Stmt::Push(val.to_string())),
                ("plant", [Sexp::Atom(Atom::S(label)), Sexp::Atom(Atom::I(n))]) => {
                    Ok(Stmt::Plant(label.to_string(), *n as u32))
                }
                ("push", _) => Err("push expects one string argument".into()),
                ("plant", _) => Err("plant expects a string label and an integer count".into()),
                (other, _) => Err(format!("unknown operator: {}", other)),
            }
        }
        [] => Err("empty list".into()),
    }
}

pub fn parse_all(src: &str) -> Result<Vec<Stmt>, String> {
    let sexp = sexp::parse(src).map_err(|e| format!("sexp parse error: {:?}", e))?;
    let list = match sexp {
        Sexp::List(sexps) => sexps,
        _ => return Err("expected a list of statements".into()),
    };
    list.into_iter().map(parse).collect()
}
