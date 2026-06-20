// Created by Sean L. on Jun. 20.
// Last Updated by Sean L. on Jun. 20.
//
// sewing-box
// packages/wasm-i8080/src/test_instructions.rs
//
// Makabaka1880, 2026. All rights reserved.

use crate::instructions::{Cc, Instr};

#[test]
fn cc_from_bits_all() {
    assert_eq!(Cc::from_bits(0), Cc::NZ);
    assert_eq!(Cc::from_bits(1), Cc::Z);
    assert_eq!(Cc::from_bits(2), Cc::NC);
    assert_eq!(Cc::from_bits(3), Cc::C);
    assert_eq!(Cc::from_bits(4), Cc::PO);
    assert_eq!(Cc::from_bits(5), Cc::PE);
    assert_eq!(Cc::from_bits(6), Cc::P);
    assert_eq!(Cc::from_bits(7), Cc::M);
    // high bits masked
    assert_eq!(Cc::from_bits(0x47), Cc::M); // 0x47 & 7 = 7
}

#[test]
fn byte_size_1_byte_instructions() {
    assert_eq!(Instr::Nop.byte_size(), 1);
    assert_eq!(Instr::Hlt.byte_size(), 1);
    assert_eq!(Instr::Mov { dst: 7, src: 0 }.byte_size(), 1);
    assert_eq!(Instr::Add { src: 0 }.byte_size(), 1);
    assert_eq!(Instr::Ret.byte_size(), 1);
    assert_eq!(Instr::RetCond { cc: Cc::Z }.byte_size(), 1);
    assert_eq!(Instr::Push { rp: 0 }.byte_size(), 1);
    assert_eq!(Instr::Pop { rp: 0 }.byte_size(), 1);
    assert_eq!(Instr::Rst { n: 7 }.byte_size(), 1);
    assert_eq!(Instr::Rlc.byte_size(), 1);
    assert_eq!(Instr::Daa.byte_size(), 1);
    assert_eq!(Instr::Xchg.byte_size(), 1);
    assert_eq!(Instr::Ei.byte_size(), 1);
    assert_eq!(Instr::Di.byte_size(), 1);
}

#[test]
fn byte_size_2_byte_instructions() {
    assert_eq!(Instr::Mvi { dst: 7, imm: 0x3E }.byte_size(), 2);
    assert_eq!(Instr::Adi { imm: 0x10 }.byte_size(), 2);
    assert_eq!(Instr::Sui { imm: 0x10 }.byte_size(), 2);
    assert_eq!(Instr::Cpi { imm: 0x10 }.byte_size(), 2);
    assert_eq!(Instr::In { port: 0x01 }.byte_size(), 2);
    assert_eq!(Instr::Out { port: 0x01 }.byte_size(), 2);
}

#[test]
fn byte_size_3_byte_instructions() {
    assert_eq!(Instr::Lxi { rp: 0, imm: 0x1234 }.byte_size(), 3);
    assert_eq!(Instr::Lda { addr: 0x1234 }.byte_size(), 3);
    assert_eq!(Instr::Sta { addr: 0x1234 }.byte_size(), 3);
    assert_eq!(Instr::Jmp { addr: 0x1234 }.byte_size(), 3);
    assert_eq!(
        Instr::Jcond {
            cc: Cc::Z,
            addr: 0x1234
        }
        .byte_size(),
        3
    );
    assert_eq!(Instr::Call { addr: 0x1234 }.byte_size(), 3);
    assert_eq!(
        Instr::CallCond {
            cc: Cc::NZ,
            addr: 0x1234
        }
        .byte_size(),
        3
    );
}
