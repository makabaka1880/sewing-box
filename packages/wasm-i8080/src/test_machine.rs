// Created by Sean L. on Jun. 20.
// Last Updated by Sean L. on Jun. 20.
//
// sewing-box
// packages/wasm-i8080/src/test_machine.rs
//
// Makabaka1880, 2026. All rights reserved.

use crate::instructions::{Cc, Instr};
use crate::machine::I8080;
use crate::machine::{A, B, C, D, E, H, L, F, BC, DE, HL, SP, PSW};

fn load(mem: &[(u16, u8)]) -> I8080 {
    let mut m = I8080::new();
    for &(addr, byte) in mem {
        m.mem[addr as usize] = byte;
    }
    m
}

fn flags(f: u8) -> [u8; 8] {
    let mut r = [0u8; 8];
    r[F as usize] = f;
    r
}

// ----------------------------------------------------------------
// register I/O
// ----------------------------------------------------------------

#[test]
fn get_set_registers() {
    let mut m = I8080::new();
    m.regs = [1, 2, 3, 4, 5, 6, 0, 8];
    assert_eq!(m.get(B), 1);
    assert_eq!(m.get(A), 8);
    m.set(8, 0xAA); // high bit masked → reg 0
    assert_eq!(m.regs[B as usize], 0xAA);
}

#[test]
fn get_set_m_indirect() {
    let mut m = I8080::new();
    m.regs[H as usize] = 0x10;
    m.regs[L as usize] = 0x00;
    m.mem[0x1000] = 0x5A;
    assert_eq!(m.get(6), 0x5A); // 6 = M
    m.set(6, 0x42);
    assert_eq!(m.mem[0x1000], 0x42);
}

#[test]
fn register_pairs() {
    let mut m = I8080::new();
    m.regs[B as usize] = 0x12;
    m.regs[C as usize] = 0x34;
    assert_eq!(m.get_pair(BC), 0x1234);

    m.set_pair(DE, 0xABCD);
    assert_eq!(m.regs[D as usize], 0xAB);
    assert_eq!(m.regs[E as usize], 0xCD);

    m.sp = 0xF000;
    assert_eq!(m.get_pair(SP), 0xF000);
    m.set_pair(SP, 0xE000);
    assert_eq!(m.sp, 0xE000);
}

// ----------------------------------------------------------------
// stack helpers
// ----------------------------------------------------------------

#[test]
fn push_pop_word() {
    let mut m = I8080::new();
    m.sp = 0x0100;
    m.push_word(0xABCD);
    assert_eq!(m.sp, 0x00FE);
    assert_eq!(m.mem[0x00FF], 0xAB);
    assert_eq!(m.mem[0x00FE], 0xCD);
    assert_eq!(m.pop_word(), 0xABCD);
    assert_eq!(m.sp, 0x0100);
}

#[test]
fn push_pop_psw() {
    let mut m = I8080::new();
    m.regs[A as usize] = 0x55;
    m.regs[F as usize] = 0x93;
    m.sp = 0x0100;
    m.push_rp(PSW);
    assert_eq!(m.mem[0x00FF], 0x55);
    assert_eq!(m.mem[0x00FE], 0x93);

    m.regs[A as usize] = 0;
    m.regs[F as usize] = 0;
    m.pop_rp(PSW);
    assert_eq!(m.regs[A as usize], 0x55);
    assert_eq!(m.regs[F as usize], 0x93);
}

// ----------------------------------------------------------------
// flags
// ----------------------------------------------------------------

#[test]
fn flag_set_basic() {
    let m = I8080 { regs: flags(0x40), ..I8080::new() }; // Z=1
    assert!(m.flag_set(Cc::Z));
    assert!(!m.flag_set(Cc::NZ));

    let m = I8080 { regs: flags(0x01), ..I8080::new() }; // C=1
    assert!(m.flag_set(Cc::C));
    assert!(!m.flag_set(Cc::NC));

    let m = I8080 { regs: flags(0x80), ..I8080::new() }; // S=1
    assert!(m.flag_set(Cc::M));
    assert!(!m.flag_set(Cc::P));

    let m = I8080 { regs: flags(0x04), ..I8080::new() }; // P=1
    assert!(m.flag_set(Cc::PE));
    assert!(!m.flag_set(Cc::PO));
}

// ----------------------------------------------------------------
// decode
// ----------------------------------------------------------------

#[test]
fn decode_nop_hlt() {
    let mut m = load(&[(0x0100, 0x00), (0x0101, 0x76)]);
    m.pc = 0x0100;
    assert_eq!(m.decode(), Some(Instr::Nop));
    assert_eq!(m.pc, 0x0101);
    assert_eq!(m.decode(), Some(Instr::Hlt));
    assert_eq!(m.pc, 0x0102);
}

#[test]
fn decode_mov() {
    let mut m = load(&[(0x0000, 0x78)]); // mov A,B
    assert_eq!(m.decode(), Some(Instr::Mov { dst: 7, src: 0 }));
}

#[test]
fn decode_immediates() {
    let mut m = load(&[(0x0000, 0x3E), (0x0001, 0x20)]);
    assert_eq!(m.decode(), Some(Instr::Mvi { dst: 7, imm: 0x20 }));

    let mut m = load(&[(0x0000, 0x01), (0x0001, 0x34), (0x0002, 0x12)]);
    assert_eq!(m.decode(), Some(Instr::Lxi { rp: BC, imm: 0x1234 }));
}

#[test]
fn decode_branches() {
    let mut m = load(&[(0x0000, 0xC3), (0x0001, 0x00), (0x0002, 0x01)]);
    assert_eq!(m.decode(), Some(Instr::Jmp { addr: 0x0100 }));

    let mut m = load(&[(0x0000, 0xCD), (0x0001, 0x34), (0x0002, 0x12)]);
    assert_eq!(m.decode(), Some(Instr::Call { addr: 0x1234 }));

    let mut m = load(&[(0x0000, 0xC9)]);
    assert_eq!(m.decode(), Some(Instr::Ret));
}

#[test]
fn decode_conditionals() {
    let mut m = load(&[(0x0000, 0xCA), (0x0001, 0x34), (0x0002, 0x12)]);
    assert_eq!(m.decode(), Some(Instr::Jcond { cc: Cc::Z, addr: 0x1234 }));
}

#[test]
fn decode_rst() {
    let mut m = load(&[(0x0000, 0xC7)]);
    assert_eq!(m.decode(), Some(Instr::Rst { n: 0 }));
    let mut m = load(&[(0x0000, 0xFF)]);
    assert_eq!(m.decode(), Some(Instr::Rst { n: 7 }));
}

#[test]
fn decode_alu_class() {
    let mut m = load(&[(0x0000, 0x80), (0x0001, 0x90), (0x0002, 0xA0)]);
    assert_eq!(m.decode(), Some(Instr::Add { src: B }));
    assert_eq!(m.decode(), Some(Instr::Sub { src: B }));
    assert_eq!(m.decode(), Some(Instr::Ana { src: B }));
}

#[test]
fn decode_unknown_returns_none_and_step_skips() {
    let mut m = load(&[(0x0000, 0x08), (0x0001, 0x10), (0x0002, 0x76)]);
    m.pc = 0;
    // decode() returns None for unknown opcodes without advancing PC
    assert_eq!(m.decode(), None);
    assert_eq!(m.pc, 0); // decode does NOT advance PC on unknown
    // step() however does advance PC by 1 on unknown and returns Ok(false)
    assert_eq!(m.step(), Ok(false));
    assert_eq!(m.pc, 1);
    assert_eq!(m.step(), Ok(false));
    assert_eq!(m.pc, 2);
    // now at HLT
    assert_eq!(m.step(), Ok(true));
    assert_eq!(m.pc, 3); // HLT decode advanced PC by 1 (from 2 to 3)
}

// ----------------------------------------------------------------
// step – arithmetic & flags (through real decode+step)
// ----------------------------------------------------------------

#[test]
fn step_add_simple() {
    let mut m = load(&[(0x0100, 0x80)]); // ADD B
    m.pc = 0x0100;
    m.regs[A as usize] = 0x10;
    m.regs[B as usize] = 0x20;
    assert!(!m.step().unwrap());
    assert_eq!(m.regs[A as usize], 0x30);
    assert!(!m.flag_set(Cc::C));
}

#[test]
fn step_add_carry_and_zero() {
    let mut m = load(&[(0x0100, 0x80)]); // ADD B
    m.pc = 0x0100;
    m.regs[A as usize] = 0x80;
    m.regs[B as usize] = 0x80;
    m.step().unwrap();
    assert_eq!(m.regs[A as usize], 0x00);
    assert!(m.flag_set(Cc::Z));
    assert!(m.flag_set(Cc::C));
    assert!(!m.flag_set(Cc::M));
}

#[test]
fn step_sub_with_borrow() {
    let mut m = load(&[(0x0000, 0x90)]); // SUB B
    m.regs[A as usize] = 0x10;
    m.regs[B as usize] = 0x20;
    m.step().unwrap();
    assert_eq!(m.regs[A as usize], 0xF0);
    assert!(m.flag_set(Cc::C));
    assert!(m.flag_set(Cc::M));
}

#[test]
fn step_cmp_preserves_a() {
    let mut m = load(&[(0x0000, 0xB8)]); // CMP B
    m.regs[A as usize] = 0x42;
    m.regs[B as usize] = 0x42;
    m.step().unwrap();
    assert_eq!(m.regs[A as usize], 0x42);
    assert!(m.flag_set(Cc::Z));
}

#[test]
fn step_adc_with_carry_in() {
    let mut m = load(&[(0x0000, 0x88)]); // ADC B
    m.regs[A as usize] = 0x01;
    m.regs[B as usize] = 0x01;
    m.regs[F as usize] = 0x01; // C=1
    m.step().unwrap();
    assert_eq!(m.regs[A as usize], 0x03); // 1+1+1
}

#[test]
fn step_inr_wrap() {
    let mut m = load(&[(0x0000, 0x3C)]); // INR A
    m.regs[A as usize] = 0xFF;
    m.step().unwrap();
    assert_eq!(m.regs[A as usize], 0x00);
    assert!(m.flag_set(Cc::Z));
    assert_eq!((m.regs[F as usize] >> 4) & 1, 1); // AC
}

#[test]
fn step_dcr_zero() {
    let mut m = load(&[(0x0000, 0x3D)]); // DCR A
    m.regs[A as usize] = 0x01;
    m.step().unwrap();
    assert_eq!(m.regs[A as usize], 0x00);
    assert!(m.flag_set(Cc::Z));
}

#[test]
fn step_inx_dcx() {
    let mut m = load(&[(0x0000, 0x23), (0x0001, 0x2B)]); // INX H, DCX H
    m.regs[H as usize] = 0xFF;
    m.regs[L as usize] = 0xFF;
    m.step().unwrap();
    assert_eq!(m.get_pair(HL), 0x0000);
    m.step().unwrap();
    assert_eq!(m.get_pair(HL), 0xFFFF);
}

#[test]
fn step_dad_no_carry() {
    let mut m = load(&[(0x0000, 0x19)]); // DAD D
    m.regs[H as usize] = 0x10;
    m.regs[L as usize] = 0x00;
    m.regs[D as usize] = 0x20;
    m.regs[E as usize] = 0x00;
    m.step().unwrap();
    assert_eq!(m.get_pair(HL), 0x3000);
    assert!(!m.flag_set(Cc::C));
}

#[test]
fn step_dad_with_carry() {
    let mut m = load(&[(0x0000, 0x09)]); // DAD B
    m.regs[H as usize] = 0xFF;
    m.regs[L as usize] = 0xFF;
    m.regs[B as usize] = 0x00;
    m.regs[C as usize] = 0x02;
    m.step().unwrap();
    assert_eq!(m.get_pair(HL), 0x0001);
    assert!(m.flag_set(Cc::C));
}

// ----------------------------------------------------------------
// step – logic
// ----------------------------------------------------------------

#[test]
fn step_ana_zero() {
    let mut m = load(&[(0x0000, 0xA0)]); // ANA B
    m.regs[A as usize] = 0xF0;
    m.regs[B as usize] = 0x0F;
    m.step().unwrap();
    assert_eq!(m.regs[A as usize], 0x00);
    assert!(m.flag_set(Cc::Z));
}

#[test]
fn step_ora() {
    let mut m = load(&[(0x0000, 0xB0)]); // ORA B
    m.regs[A as usize] = 0x00;
    m.regs[B as usize] = 0x01;
    m.step().unwrap();
    assert_eq!(m.regs[A as usize], 0x01);
    assert!(!m.flag_set(Cc::Z));
}

#[test]
fn step_xra_clears_carry() {
    let mut m = load(&[(0x0000, 0xAF)]); // XRA A
    m.regs[F as usize] = 0x01;
    m.regs[A as usize] = 0x55;
    m.step().unwrap();
    assert_eq!(m.regs[A as usize], 0x00);
    assert!(m.flag_set(Cc::Z));
    assert!(!m.flag_set(Cc::C));
}

// ----------------------------------------------------------------
// step – rotates
// ----------------------------------------------------------------

#[test]
fn step_rlc() {
    let mut m = load(&[(0x0000, 0x07)]);
    m.regs[A as usize] = 0x81;
    m.step().unwrap();
    assert_eq!(m.regs[A as usize], 0x03);
    assert!(m.flag_set(Cc::C));
}

#[test]
fn step_rrc() {
    let mut m = load(&[(0x0000, 0x0F)]);
    m.regs[A as usize] = 0x01;
    m.step().unwrap();
    assert_eq!(m.regs[A as usize], 0x80);
    assert!(m.flag_set(Cc::C));
}

#[test]
fn step_ral_rvc() {
    let mut m = load(&[(0x0000, 0x17)]);
    m.regs[A as usize] = 0x81;
    m.regs[F as usize] = 0x00;
    m.step().unwrap();
    assert_eq!(m.regs[A as usize], 0x02);
    assert!(m.flag_set(Cc::C));
}

#[test]
fn step_rar() {
    let mut m = load(&[(0x0000, 0x1F)]);
    m.regs[A as usize] = 0x01;
    m.regs[F as usize] = 0x01;
    m.step().unwrap();
    assert_eq!(m.regs[A as usize], 0x80);
    assert!(m.flag_set(Cc::C));
}

// ----------------------------------------------------------------
// step – control flow
// ----------------------------------------------------------------

#[test]
fn step_jmp() {
    let mut m = load(&[(0x0100, 0xC3), (0x0101, 0x00), (0x0102, 0x02)]);
    m.pc = 0x0100;
    m.step().unwrap();
    assert_eq!(m.pc, 0x0200);
}

#[test]
fn step_jcond_taken() {
    let mut m = load(&[(0x0000, 0xDA), (0x0001, 0x00), (0x0002, 0x02)]); // jc 0200h
    m.regs[F as usize] = 0x01; // C=1
    m.step().unwrap();
    assert_eq!(m.pc, 0x0200);
}

#[test]
fn step_jcond_not_taken() {
    let mut m = load(&[(0x0000, 0xDA), (0x0001, 0x00), (0x0002, 0x02)]); // jc 0200h
    m.regs[F as usize] = 0x00; // C=0
    m.step().unwrap();
    assert_eq!(m.pc, 3); // fell through
}

#[test]
fn step_call_ret() {
    let mut m = load(&[
        (0x0100, 0xCD), (0x0101, 0x00), (0x0102, 0x02), // call 0200h
        (0x0200, 0xC9), // ret
    ]);
    m.pc = 0x0100;
    m.sp = 0x0100;
    m.step().unwrap();
    assert_eq!(m.pc, 0x0200);
    m.step().unwrap();
    assert_eq!(m.pc, 0x0103); // back after call
}

#[test]
fn step_rst() {
    let mut m = load(&[(0x0100, 0xCF)]); // RST 1
    m.pc = 0x0100;
    m.sp = 0x0100;
    m.step().unwrap();
    assert_eq!(m.pc, 0x0008);
    assert_eq!(m.pop_word(), 0x0101);
}

#[test]
fn step_hlt() {
    let mut m = load(&[(0x0000, 0x76)]);
    assert_eq!(m.step().unwrap(), true);
}

// ----------------------------------------------------------------
// step – stack ops
// ----------------------------------------------------------------

#[test]
fn step_push_pop() {
    let mut m = load(&[(0x0000, 0xC5), (0x0001, 0xD1)]); // push B, pop D
    m.sp = 0x0100;
    m.regs[B as usize] = 0x12;
    m.regs[C as usize] = 0x34;
    m.step().unwrap();
    assert_eq!(m.sp, 0x00FE);
    m.step().unwrap();
    assert_eq!(m.regs[D as usize], 0x12);
    assert_eq!(m.regs[E as usize], 0x34);
}

#[test]
fn step_xchg() {
    let mut m = load(&[(0x0000, 0xEB)]);
    m.regs[D as usize] = 0x11;
    m.regs[E as usize] = 0x22;
    m.regs[H as usize] = 0xAA;
    m.regs[L as usize] = 0xBB;
    m.step().unwrap();
    assert_eq!(m.regs[D as usize], 0xAA);
    assert_eq!(m.regs[H as usize], 0x11);
}

#[test]
fn step_sphl() {
    let mut m = load(&[(0x0000, 0xF9)]);
    m.regs[H as usize] = 0x0F;
    m.regs[L as usize] = 0x00;
    m.step().unwrap();
    assert_eq!(m.sp, 0x0F00);
}

#[test]
fn step_xthl() {
    let mut m = load(&[(0x0000, 0xE3)]);
    m.sp = 0x0100;
    m.mem[0x0100] = 0x34;
    m.mem[0x0101] = 0x12;
    m.regs[H as usize] = 0xAB;
    m.regs[L as usize] = 0xCD;
    m.step().unwrap();
    assert_eq!(m.get_pair(HL), 0x1234);
    assert_eq!(m.mem[0x0100], 0xCD);
    assert_eq!(m.mem[0x0101], 0xAB);
}

// ----------------------------------------------------------------
// step – I/O
// ----------------------------------------------------------------

#[test]
fn step_out_in() {
    let mut m = load(&[(0x0000, 0xD3), (0x0001, 0x10), (0x0002, 0xDB), (0x0003, 0x10)]);
    m.regs[A as usize] = 0x5A;
    m.step().unwrap(); // out 10h
    assert_eq!(m.ports[0x10], 0x5A);

    m.regs[A as usize] = 0;
    m.step().unwrap(); // in 10h
    assert_eq!(m.regs[A as usize], 0x5A);
}

// ----------------------------------------------------------------
// step – LHLD / SHLD byte order (little-endian)
// ----------------------------------------------------------------

#[test]
fn step_lhld_byte_order() {
    // LHLD loads L from [addr], H from [addr+1]
    let mut m = load(&[(0x0000, 0x2A), (0x0001, 0x00), (0x0002, 0x10)]);
    m.mem[0x1000] = 0x34; // low byte
    m.mem[0x1001] = 0x12; // high byte
    m.step().unwrap();
    assert_eq!(m.regs[L as usize], 0x34, "L should be low byte");
    assert_eq!(m.regs[H as usize], 0x12, "H should be high byte");
    assert_eq!(m.get_pair(HL), 0x1234);
}

#[test]
fn step_shld_byte_order() {
    // SHLD stores L to [addr], H to [addr+1]
    let mut m = load(&[(0x0000, 0x22), (0x0001, 0x00), (0x0002, 0x10)]);
    m.regs[L as usize] = 0x78;
    m.regs[H as usize] = 0x56;
    m.step().unwrap();
    assert_eq!(m.mem[0x1000], 0x78, "low byte should be L");
    assert_eq!(m.mem[0x1001], 0x56, "high byte should be H");
}

// ----------------------------------------------------------------
// step – CMP / CPI aux carry
// ----------------------------------------------------------------

#[test]
fn step_cmp_aux_carry_no_borrow() {
    // CMP: AC=1 when borrow occurs from bit 3 (same as SUB)
    // A=0x1F, B=0x0A: low nibbles 0xF - 0xA = 0x5, no borrow → AC=0
    let mut m = load(&[(0x0000, 0xB8)]); // CMP B
    m.regs[A as usize] = 0x1F;
    m.regs[B as usize] = 0x0A;
    m.step().unwrap();
    assert_eq!((m.regs[F as usize] >> 4) & 1, 0, "AC should be clear: low nibble 0xF >= 0xA, no borrow");
}

#[test]
fn step_cmp_aux_carry_with_borrow() {
    // CMP: AC=1 when borrow occurs from bit 3
    // A=0x10, B=0x01: low nibbles 0x0 - 0x1, borrow → AC=1
    let mut m = load(&[(0x0000, 0xB8)]); // CMP B
    m.regs[A as usize] = 0x10;
    m.regs[B as usize] = 0x01;
    m.step().unwrap();
    assert_eq!((m.regs[F as usize] >> 4) & 1, 1, "AC should be set when low nibble A < low nibble src");
}

#[test]
fn step_cmp_aux_carry_high_nibble_does_not_mask() {
    // Regression: old code used a.overflowing_sub(val & 0x0F) which
    // could mask a borrow when A's high nibble contributed carry bits.
    // Example: A=0x20, val=0x0F → A & 0x0F = 0x0, val & 0x0F = 0xF
    // 0x0 - 0xF → borrow → AC=1 (correct). The old code did
    // 0x20.overflowing_sub(0x0F) = 0x11, no borrow → AC=0 (wrong).
    // So the bug was that AC was incorrectly 0 instead of 1.
    let mut m = load(&[(0x0000, 0xB8)]); // CMP B
    m.regs[A as usize] = 0x20;
    m.regs[B as usize] = 0x0F;
    m.step().unwrap();
    assert_eq!((m.regs[F as usize] >> 4) & 1, 1, "AC should be set: low nibble 0x0 < 0xF causes borrow");
}

#[test]
fn step_cpi_aux_carry_regression() {
    // Same regression check for CPI
    let mut m = load(&[(0x0000, 0xFE), (0x0001, 0x0F)]); // CPI 0Fh
    m.regs[A as usize] = 0x20;
    m.step().unwrap();
    assert_eq!((m.regs[F as usize] >> 4) & 1, 1, "AC should be set: low nibble 0x0 < 0xF causes borrow");
}

// ----------------------------------------------------------------
// step – EI / DI interrupt enable flag
// ----------------------------------------------------------------

#[test]
fn step_ei_sets_int_enable() {
    let mut m = load(&[(0x0000, 0xFB)]); // EI
    assert!(!m.int_enable);
    m.step().unwrap();
    assert!(m.int_enable);
}

#[test]
fn step_di_clears_int_enable() {
    let mut m = load(&[(0x0000, 0xFB), (0x0001, 0xF3)]); // EI, DI
    m.step().unwrap();
    assert!(m.int_enable);
    m.step().unwrap();
    assert!(!m.int_enable);
}

// ----------------------------------------------------------------
// step – M (indirect via HL)
// ----------------------------------------------------------------

#[test]
fn step_mov_to_m() {
    let mut m = load(&[(0x0000, 0x70)]); // mov M,B
    m.regs[H as usize] = 0x20;
    m.regs[L as usize] = 0x00;
    m.regs[B as usize] = 0x42;
    m.step().unwrap();
    assert_eq!(m.mem[0x2000], 0x42);
}

#[test]
fn step_mov_from_m() {
    let mut m = load(&[(0x0000, 0x46)]); // mov B,M
    m.regs[H as usize] = 0x20;
    m.regs[L as usize] = 0x00;
    m.mem[0x2000] = 0x77;
    m.step().unwrap();
    assert_eq!(m.regs[B as usize], 0x77);
}

// ----------------------------------------------------------------
// step – DAA
// ----------------------------------------------------------------

#[test]
fn step_daa_basic() {
    let mut m = load(&[(0x0000, 0x27)]);
    m.regs[A as usize] = 0x0A;
    m.step().unwrap();
    assert_eq!(m.regs[A as usize], 0x10);
}

#[test]
fn step_daa_with_carry() {
    let mut m = load(&[(0x0000, 0x27)]);
    m.regs[A as usize] = 0x9A;
    m.regs[F as usize] = 0x01;
    m.step().unwrap();
    assert_eq!(m.regs[A as usize], 0x00);
    assert!(m.flag_set(Cc::C));
}

// ----------------------------------------------------------------
// disasm
// ----------------------------------------------------------------

#[test]
fn disasm_various() {
    let m = load(&[
        (0x0000, 0x78), (0x0001, 0x3E), (0x0002, 0x20),
        (0x0003, 0x01), (0x0004, 0x34), (0x0005, 0x12),
        (0x0006, 0xC3), (0x0007, 0x00), (0x0008, 0x01),
        (0x0009, 0xC5), (0x000A, 0xC1), (0x000B, 0xCA),
        (0x000C, 0x34), (0x000D, 0x12), (0x000E, 0xC7),
        (0x000F, 0xC8), (0x0010, 0xFF),
    ]);
    assert_eq!(m.disasm_at(0x0000), "(mov A B)");
    assert_eq!(m.disasm_at(0x0001), "(mvi A 20h)");
    assert_eq!(m.disasm_at(0x0003), "(lxi B 1234h)");
    assert_eq!(m.disasm_at(0x0006), "(jmp 0100h)");
    assert_eq!(m.disasm_at(0x0009), "(push B)");
    assert_eq!(m.disasm_at(0x000A), "(pop B)");
    assert_eq!(m.disasm_at(0x000B), "(jcond Z 1234h)");
    assert_eq!(m.disasm_at(0x000E), "(rst 0)");
    assert_eq!(m.disasm_at(0x000F), "(retCond Z)");
    assert_eq!(m.disasm_at(0x0010), "(rst 7)");
}
