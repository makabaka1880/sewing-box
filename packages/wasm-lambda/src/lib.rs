// Created by Sean L. on Jun. 17.
// Last Updated by Sean L. on Jun. 17.
//
// wasm-lambda
// src/lib.rs
//
// Makabaka1880, 2026. All rights reserved.

use crate::{interpreter::KrivMachine, parser::parse};
use serde_wasm_bindgen;
use wasm_bindgen::prelude::*;

pub mod interpreter;
pub mod machine;
pub mod parser;

#[wasm_bindgen]
pub struct LambdaProgram {
    inner: KrivMachine,
}

#[wasm_bindgen]
impl LambdaProgram {
    #[wasm_bindgen(constructor)]
    pub fn new(src: &str) -> Result<LambdaProgram, JsValue> {
        let sexp = match sexp::parse(src) {
            Ok(e) => e,
            Err(e) => return Err(JsValue::from_str(&e.to_string())),
        };
        match parse(sexp) {
            Ok(t) => Ok(LambdaProgram {
                inner: KrivMachine::new(t),
            }),
            Err(e) => Err(JsValue::from_str(&e)),
        }
    }

    #[wasm_bindgen]
    pub fn step(&mut self) -> Result<bool, JsValue> {
        self.inner.step().map_err(|e| JsValue::from_str(&e))
    }

    #[wasm_bindgen]
    pub fn state(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.inner).unwrap_or(JsValue::UNDEFINED)
    }
}
