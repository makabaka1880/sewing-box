// Created by Sean L. on Jun. 20.
// Last Updated by Sean L. on Jun. 20.
//
// sewing-box
// packages/wasm-i8080/src/test_parser.rs
//
// Makabaka1880, 2026. All rights reserved.

use crate::instructions::{Cc, Instr};
use crate::parser::{parse, Data};

#[test]
fn parse_example_from_readme() {
    let src = r#"
(block 0100h
  (mov A B)
  (add A)
  (jmp 0105h))

(block 0105h
  (db 'H' 'i' 00h)
  (hlt))
"#;
    let blocks = parse(src).unwrap();
    assert_eq!(blocks.len(), 2);

    assert_eq!(blocks[0].anchor, 0x0100);
    assert_eq!(blocks[0].content.len(), 3);
    assert!(matches!(
        blocks[0].content[0],
        Data::Code(Instr::Mov { dst: 7, src: 0 })
    ));
    assert!(matches!(
        blocks[0].content[1],
        Data::Code(Instr::Add { src: 7 })
    ));
    assert!(matches!(
        blocks[0].content[2],
        Data::Code(Instr::Jmp { addr: 0x0105 })
    ));

    assert_eq!(blocks[1].anchor, 0x0105);
    assert_eq!(blocks[1].content.len(), 2);
    assert!(matches!(blocks[1].content[0], Data::Bytes(ref b) if b == &[b'H', b'i', 0x00]));
    assert!(matches!(blocks[1].content[1], Data::Code(Instr::Hlt)));
}

#[test]
fn empty_input() {
    let blocks = parse("").unwrap();
    assert!(blocks.is_empty());
}

#[test]
fn empty_block() {
    let blocks = parse("(block 0100h)").unwrap();
    assert_eq!(blocks.len(), 1);
    assert_eq!(blocks[0].anchor, 0x0100);
    assert!(blocks[0].content.is_empty());
}

#[test]
fn decimal_address() {
    let blocks = parse("(block 256 (nop))").unwrap();
    assert_eq!(blocks[0].anchor, 256);
}

#[test]
fn label_is_error() {
    let src = "(block FOO (nop))";
    assert!(parse(src).is_err());
}

#[test]
fn all_registers() {
    let src = "(block 0 (mov A A) (mov B C) (mov D E) (mov H L) (mov M A))";
    let blocks = parse(src).unwrap();
    assert_eq!(blocks[0].content.len(), 5);
    assert!(matches!(
        blocks[0].content[0],
        Data::Code(Instr::Mov { dst: 7, src: 7 })
    ));
    assert!(matches!(
        blocks[0].content[1],
        Data::Code(Instr::Mov { dst: 0, src: 1 })
    ));
    assert!(matches!(
        blocks[0].content[2],
        Data::Code(Instr::Mov { dst: 2, src: 3 })
    ));
    assert!(matches!(
        blocks[0].content[3],
        Data::Code(Instr::Mov { dst: 4, src: 5 })
    ));
    assert!(matches!(
        blocks[0].content[4],
        Data::Code(Instr::Mov { dst: 6, src: 7 })
    ));
}

#[test]
fn register_pairs() {
    let src = "(block 0 (lxi B 1234h) (push PSW) (pop SP) (inx D) (dcx H))";
    let blocks = parse(src).unwrap();
    assert_eq!(blocks[0].content.len(), 5);
    assert!(matches!(
        blocks[0].content[0],
        Data::Code(Instr::Lxi { rp: 0, imm: 0x1234 })
    ));
    assert!(matches!(
        blocks[0].content[1],
        Data::Code(Instr::Push { rp: 3 })
    ));
    assert!(matches!(
        blocks[0].content[2],
        Data::Code(Instr::Pop { rp: 3 })
    ));
    assert!(matches!(
        blocks[0].content[3],
        Data::Code(Instr::Inx { rp: 1 })
    ));
    assert!(matches!(
        blocks[0].content[4],
        Data::Code(Instr::Dcx { rp: 2 })
    ));
}

#[test]
fn immediate_byte_instructions() {
    let src = "(block 0 (mvi A 3Eh) (adi 10h) (sui 5) (cpi 'X'))";
    let blocks = parse(src).unwrap();
    assert_eq!(blocks[0].content.len(), 4);
    assert!(matches!(
        blocks[0].content[0],
        Data::Code(Instr::Mvi { dst: 7, imm: 0x3E })
    ));
    assert!(matches!(
        blocks[0].content[1],
        Data::Code(Instr::Adi { imm: 0x10 })
    ));
    assert!(matches!(
        blocks[0].content[2],
        Data::Code(Instr::Sui { imm: 5 })
    ));
    assert!(matches!(
        blocks[0].content[3],
        Data::Code(Instr::Cpi { imm: b'X' })
    ));
}

#[test]
fn conditional_branches() {
    let src = "(block 0 (jnz 0100h) (cz 0200h) (rnc) (jcond Z 0300h) (callCond NC 0400h) (retCond PE))";
    let blocks = parse(src).unwrap();
    assert!(matches!(
        blocks[0].content[0],
        Data::Code(Instr::Jcond { cc: Cc::NZ, addr: 0x0100 })
    ));
    assert!(matches!(
        blocks[0].content[1],
        Data::Code(Instr::CallCond { cc: Cc::Z, addr: 0x0200 })
    ));
    assert!(matches!(
        blocks[0].content[2],
        Data::Code(Instr::RetCond { cc: Cc::NC })
    ));
    assert!(matches!(
        blocks[0].content[3],
        Data::Code(Instr::Jcond { cc: Cc::Z, addr: 0x0300 })
    ));
    assert!(matches!(
        blocks[0].content[4],
        Data::Code(Instr::CallCond { cc: Cc::NC, addr: 0x0400 })
    ));
    assert!(matches!(
        blocks[0].content[5],
        Data::Code(Instr::RetCond { cc: Cc::PE })
    ));
}

#[test]
fn rst_vector() {
    let src = "(block 0 (rst 0) (rst 7))";
    let blocks = parse(src).unwrap();
    assert!(matches!(
        blocks[0].content[0],
        Data::Code(Instr::Rst { n: 0 })
    ));
    assert!(matches!(
        blocks[0].content[1],
        Data::Code(Instr::Rst { n: 7 })
    ));
}

#[test]
fn rst_out_of_range() {
    let src = "(block 0 (rst 8))";
    assert!(parse(src).is_err());
}

#[test]
fn io_ports() {
    let src = "(block 0 (in 10h) (out 0FFh))";
    let blocks = parse(src).unwrap();
    assert!(matches!(
        blocks[0].content[0],
        Data::Code(Instr::In { port: 0x10 })
    ));
    assert!(matches!(
        blocks[0].content[1],
        Data::Code(Instr::Out { port: 0xFF })
    ));
}

#[test]
fn db_character_literals() {
    let src = "(block 0 (db 'H' 'e' 'l' 'l' 'o'))";
    let blocks = parse(src).unwrap();
    assert!(matches!(blocks[0].content[0], Data::Bytes(ref b) if b == b"Hello"));
}

#[test]
fn unknown_mnemonic_is_error() {
    let src = "(block 0 (frobnicate))";
    assert!(parse(src).is_err());
}

#[test]
fn wrong_arg_count_is_error() {
    let src = "(block 0 (mov A))";
    assert!(parse(src).is_err());
}
