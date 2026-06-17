// Created by Sean L. on Jun. 17.
// Last Updated by Sean L. on Jun. 17.
//
// sewing-box
// packages/wasm-stacky/src/data.rs
//
// Makabaka1880, 2026. All rights reserved.

use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub enum Tree {
    Atom(String),
    Branch(String, Vec<Tree>),
}
