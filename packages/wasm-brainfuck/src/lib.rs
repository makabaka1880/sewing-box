// Created by Sean L. on Jun. 19.
// Last Updated by Sean L. on Jun. 19.
//
// sewing-box
// packages/brainfuck-wasm/src/lib.rs
//
// Makabaka1880, 2026. All rights reserved.

use wasm_bindgen::prelude::*;

mod interpreter;
mod parser;

use crate::{interpreter::*, parser::*};

#[wasm_bindgen]
pub struct BFProgramWasm {
    inner: BFMachine,
}

#[wasm_bindgen]
impl BFProgramWasm {
    #[wasm_bindgen(constructor)]
    pub fn new(src: &str) -> Result<BFProgramWasm, JsValue> {
        BFMachine::new(parse(src))
            .map_err(|e| JsValue::from_str(&e))
            .map(|prog| BFProgramWasm { inner: prog })
    }

    #[wasm_bindgen]
    pub fn tape_slice(&self, from: i32, to: i32) -> Vec<i32> {
        self.inner.tape_slice(from, to)
    }

    #[wasm_bindgen]
    pub fn feed_input(&mut self, input: &[i32]) {
        self.inner.feed_input(input);
    }

    #[wasm_bindgen]
    pub fn input_len(&self) -> usize {
        self.inner.input_len()
    }

    #[wasm_bindgen]
    pub fn output(&self) -> Vec<i32> {
        self.inner.output().to_vec()
    }

    #[wasm_bindgen]
    pub fn tape_ptr(&self) -> i32 {
        self.inner.tape_ptr()
    }

    #[wasm_bindgen]
    pub fn prog_ptr(&self) -> usize {
        self.inner.prog_ptr()
    }

    #[wasm_bindgen]
    pub fn instr_at(&self, pos: usize) -> Option<String> {
        self.inner.instr_at(pos)
    }

    #[wasm_bindgen]
    pub fn step(&mut self) -> Result<bool, JsValue> {
        self.inner.step().map_err(|e| JsValue::from_str(&e))
    }
}
