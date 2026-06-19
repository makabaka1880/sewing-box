// Created by Sean L. on Jun. 17.
// Last Updated by Sean L. on Jun. 17.
//
// sewing-box
// packages/wasm-stacky/src/lib.rs
//
// Makabaka1880, 2026. All rights reserved.

use interpreter::StackyProgram;
use parser::parse_all;
use serde_wasm_bindgen;
use wasm_bindgen::prelude::*;

pub mod data;
pub mod interpreter;
pub mod parser;

#[wasm_bindgen]
pub struct StackyProgramWasm {
    inner: StackyProgram,
}

#[wasm_bindgen]
impl StackyProgramWasm {
    #[wasm_bindgen(constructor)]
    pub fn new(src: &str) -> Result<StackyProgramWasm, JsValue> {
        let stmts = parse_all(src).map_err(|e| JsValue::from_str(&e))?;
        Ok(StackyProgramWasm {
            inner: StackyProgram::new(stmts),
        })
    }

    #[wasm_bindgen]
    pub fn step(&mut self) -> Result<bool, JsValue> {
        self.inner.step().map_err(|e| JsValue::from_str(&e))
    }

    #[wasm_bindgen]
    pub fn state(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.inner.state()).unwrap_or(JsValue::UNDEFINED)
    }
}
