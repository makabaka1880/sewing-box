// Created by Sean L. on Jun. 20.
// Last Updated by Sean L. on Jun. 20.
//
// sewing-box
// packages/wasm-i8080/src/lib.rs
//
// Makabaka1880, 2026. All rights reserved.

pub mod assembler;
pub mod instructions;
pub mod machine;
pub mod parser;

#[cfg(test)]
mod test_instructions;
#[cfg(test)]
mod test_machine;
#[cfg(test)]
mod test_parser;

use machine::I8080;
use parser::parse;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct I8080Wasm {
    inner: I8080,
    halted: bool,
}

#[wasm_bindgen]
impl I8080Wasm {
    #[wasm_bindgen(constructor)]
    pub fn new(src: &str, io_bus: Vec<u8>, pc: u16, sp: u16) -> Result<I8080Wasm, JsValue> {
        let blocks = parse(src).map_err(|e| JsValue::from_str(&e))?;
        let inner = I8080::assemble(blocks, io_bus, pc, sp).map_err(|e| JsValue::from_str(&e))?;
        Ok(I8080Wasm {
            inner,
            halted: false,
        })
    }

    #[wasm_bindgen]
    pub fn step(&mut self) -> Result<bool, JsValue> {
        if self.halted {
            return Ok(true);
        }
        let halted = self.inner.step().map_err(|e| JsValue::from_str(&e))?;
        self.halted = halted;
        Ok(halted)
    }

    #[wasm_bindgen]
    pub fn state(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&self.inner.state(self.halted)).unwrap_or(JsValue::UNDEFINED)
    }

    #[wasm_bindgen]
    pub fn regs(&self) -> Vec<u8> {
        self.inner.regs.to_vec()
    }

    #[wasm_bindgen]
    pub fn set_regs(&mut self, val: Vec<u8>) {
        let len = val.len().min(8);
        self.inner.regs[..len].copy_from_slice(&val[..len]);
    }

    #[wasm_bindgen]
    pub fn pc(&self) -> u16 {
        self.inner.pc
    }

    #[wasm_bindgen]
    pub fn set_pc(&mut self, val: u16) {
        self.inner.pc = val;
    }

    #[wasm_bindgen]
    pub fn sp(&self) -> u16 {
        self.inner.sp
    }

    #[wasm_bindgen]
    pub fn set_sp(&mut self, val: u16) {
        self.inner.sp = val;
    }

    #[wasm_bindgen]
    pub fn disasm(&self) -> String {
        self.inner.disasm_at(self.inner.pc)
    }

    #[wasm_bindgen]
    pub fn instr_bytes(&self) -> Vec<u8> {
        let addr = self.inner.pc as usize;
        if addr + 3 > 0x10000 {
            return self.inner.mem[addr..].to_vec();
        }
        self.inner.mem[addr..addr + 3].to_vec()
    }

    #[wasm_bindgen]
    pub fn ports(&self) -> Vec<u8> {
        self.inner.ports.to_vec()
    }

    #[wasm_bindgen]
    pub fn set_ports(&mut self, val: Vec<u8>) {
        let len = val.len().min(256);
        self.inner.ports[..len].copy_from_slice(&val[..len]);
    }

    #[wasm_bindgen]
    pub fn int_enabled(&self) -> bool {
        self.inner.int_enable
    }

    #[wasm_bindgen]
    pub fn set_int_enabled(&mut self, val: bool) {
        self.inner.int_enable = val;
    }

    #[wasm_bindgen]
    pub fn memory_slice(&self, from: usize, to: usize) -> Vec<u8> {
        let end = to.min(0x10000);
        if from >= end {
            return Vec::new();
        }
        self.inner.mem[from..end].to_vec()
    }

    #[wasm_bindgen]
    pub fn memory_read_byte(&self, addr: usize) -> u8 {
        if addr < 0x10000 {
            self.inner.mem[addr]
        } else {
            0
        }
    }

    #[wasm_bindgen]
    pub fn memory_write_byte(&mut self, addr: usize, value: u8) {
        if addr < 0x10000 {
            self.inner.mem[addr] = value;
        }
    }

    #[wasm_bindgen]
    pub fn get_memory_ptr(&self) -> *const u8 {
        self.inner.mem.as_ptr()
    }
}
