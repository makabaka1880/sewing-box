// Created by Sean L. on Jun. 17.
// Last Updated by Sean L. on Jun. 17.
//
// sewing-box
// packages/wasm-stacky/src/interpreter.rs
//
// Makabaka1880, 2026. All rights reserved.

use serde::Serialize;
use crate::{data::Tree, parser::Stmt};

pub struct StackyProgram {
    commands: Vec<Stmt>,
    store: Vec<Tree>,
}

impl StackyProgram {
    pub fn new(commands: Vec<Stmt>) -> Self {
        StackyProgram {
            commands,
            store: Vec::new(),
        }
    }

    pub fn step(&mut self) -> Result<bool, String> {
        if self.commands.is_empty() {
            return Ok(true);
        }
        match self.commands.remove(0) {
            Stmt::Push(s) => self.store.push(Tree::Atom(s)),
            Stmt::Plant(s, n) => {
                let n = n as usize;
                if self.store.len() < n {
                    return Err(format!(
                        "Plant '{}' needs {} element{}, but stack has only {}",
                        s,
                        n,
                        if n == 1 { "" } else { "s" },
                        self.store.len(),
                    ));
                }
                let at = self.store.len() - n;
                let children: Vec<Tree> = self.store.drain(at..).collect();
                self.store.push(Tree::Branch(s, children));
            }
        }
        Ok(self.commands.is_empty())
    }

    pub fn state(&self) -> impl Serialize + '_ {
        #[derive(Serialize)]
        struct State<'a> {
            remaining: usize,
            stack: &'a [Tree],
        }
        State {
            remaining: self.commands.len(),
            stack: &self.store,
        }
    }
}
