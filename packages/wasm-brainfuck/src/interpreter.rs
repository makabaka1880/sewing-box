// Created by Sean L. on Jun. 18.
// Last Updated by Sean L. on Jun. 18.
//
// brainfuck-wasm
// src/interpreter.rs
//
// Makabaka1880, 2026. All rights reserved.

use crate::parser::Tokens;

pub struct Tape {
    l_tail: Vec<i32>,
    r_tail: Vec<i32>,
}

impl Tape {
    fn extend(&mut self, idx: i32) {
        if idx < 0 {
            let required = (1 - idx) as usize;
            if required >= self.l_tail.len() {
                self.l_tail.resize(2 * required.max(1), 0);
            }
        } else {
            let required = idx as usize;
            if required >= self.r_tail.len() {
                self.r_tail.resize(2 * required.max(1), 0);
            }
        }
    }
    pub fn get(&mut self, idx: i32) -> i32 {
        self.extend(idx);
        if idx < 0 {
            self.l_tail[(1 - idx) as usize]
        } else {
            self.r_tail[idx as usize]
        }
    }
    pub fn set(&mut self, idx: i32, value: i32) {
        self.extend(idx);
        if idx < 0 {
            self.l_tail[(1 - idx) as usize] = value
        } else {
            self.r_tail[idx as usize] = value
        }
    }
}

pub struct BFMachine {
    program: Vec<Tokens>,
    prog_ptr: usize,
    io_out: Vec<i32>,
    io_in: Vec<i32>,
    tape_ptr: i32,
    tape: Tape,
}

impl BFMachine {
    pub fn new(program: Vec<Tokens>) -> Result<Self, String> {
        let mut stack = Vec::new();
        for (i, tok) in program.iter().enumerate() {
            match tok {
                Tokens::BrLeft => stack.push(i),
                Tokens::PrRight => {
                    if stack.pop().is_none() {
                        return Err(format!("Unmatched ']' at position {}", i));
                    }
                }
                _ => {}
            }
        }
        if let Some(pos) = stack.pop() {
            return Err(format!("Unmatched '[' at position {}", pos));
        }
        Ok(BFMachine {
            program,
            prog_ptr: 0,
            io_out: Vec::new(),
            io_in: Vec::new(),
            tape_ptr: 0,
            tape: Tape {
                l_tail: Vec::new(),
                r_tail: Vec::new(),
            },
        })
    }

    fn scan_fwd_to_match(&self, start: usize) -> usize {
        let mut depth = 1u32;
        let mut i = start + 1;
        while i < self.program.len() {
            match &self.program[i] {
                Tokens::BrLeft => depth += 1,
                Tokens::PrRight => {
                    depth -= 1;
                    if depth == 0 {
                        return i;
                    }
                }
                _ => {}
            }
            i += 1;
        }
        start
    }

    fn scan_bwd_to_match(&self, start: usize) -> usize {
        let mut depth = 1u32;
        let mut i = start;
        while i > 0 {
            i -= 1;
            match &self.program[i] {
                Tokens::PrRight => depth += 1,
                Tokens::BrLeft => {
                    depth -= 1;
                    if depth == 0 {
                        return i;
                    }
                }
                _ => {}
            }
        }
        start
    }

    pub fn feed_input(&mut self, input: &[i32]) {
        for &ch in input.iter().rev() {
            self.io_in.push(ch);
        }
    }

    pub fn input_len(&self) -> usize {
        self.io_in.len()
    }

    pub fn output(&self) -> &[i32] {
        &self.io_out
    }

    pub fn tape_ptr(&self) -> i32 {
        self.tape_ptr
    }

    pub fn prog_ptr(&self) -> usize {
        self.prog_ptr
    }

    pub fn instr_at(&self, pos: usize) -> Option<String> {
        self.program.get(pos).map(|tok| {
            match tok {
                Tokens::IncCell => "+",
                Tokens::DecCell => "-",
                Tokens::IncPtr => ">",
                Tokens::DecPtr => "<",
                Tokens::BrLeft => "[",
                Tokens::PrRight => "]",
                Tokens::Print => ".",
                Tokens::Read => ",",
            }
            .to_string()
        })
    }

    pub fn tape_slice(&self, from: i32, to: i32) -> Vec<i32> {
        (from..to)
            .map(|idx| {
                if idx < 0 {
                    let i = (1 - idx) as usize;
                    self.tape.l_tail.get(i).copied().unwrap_or(0)
                } else {
                    let i = idx as usize;
                    self.tape.r_tail.get(i).copied().unwrap_or(0)
                }
            })
            .collect()
    }

    pub fn step(&mut self) -> Result<bool, String> {
        if self.prog_ptr >= self.program.len() {
            return Ok(true);
        }
        match &self.program[self.prog_ptr] {
            Tokens::IncCell => {
                let old_val = self.tape.get(self.tape_ptr);
                self.tape.set(self.tape_ptr, old_val + 1);
                self.prog_ptr += 1;
                Ok(false)
            }
            Tokens::DecCell => {
                let old_val = self.tape.get(self.tape_ptr);
                self.tape.set(self.tape_ptr, old_val - 1);
                self.prog_ptr += 1;
                Ok(false)
            }
            Tokens::IncPtr => {
                self.tape_ptr += 1;
                self.prog_ptr += 1;
                Ok(false)
            }
            Tokens::DecPtr => {
                self.tape_ptr -= 1;
                self.prog_ptr += 1;
                Ok(false)
            }
            Tokens::BrLeft => {
                if self.tape.get(self.tape_ptr) == 0 {
                    let jmp = self.scan_fwd_to_match(self.prog_ptr);
                    self.prog_ptr = jmp + 1;
                } else {
                    self.prog_ptr += 1;
                }
                Ok(false)
            }
            Tokens::PrRight => {
                if self.tape.get(self.tape_ptr) != 0 {
                    let jmp = self.scan_bwd_to_match(self.prog_ptr);
                    self.prog_ptr = jmp + 1;
                } else {
                    self.prog_ptr += 1;
                }
                Ok(false)
            }
            Tokens::Print => {
                self.io_out.push(self.tape.get(self.tape_ptr));
                self.prog_ptr += 1;
                Ok(false)
            }
            Tokens::Read => match self.io_in.pop() {
                Some(val) => {
                    self.tape.set(self.tape_ptr, val);
                    self.prog_ptr += 1;
                    Ok(false)
                }
                None => Err(String::from("Empty input")),
            },
        }
    }
}
