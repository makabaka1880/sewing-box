use serde::Serialize;

use crate::instructions::{
    Cc,
    Instr::{self, *},
};

// Created by Sean L. on Jun. 20.
// Last Updated by Sean L. on Jun. 20.
//
// sewing-box
// packages/wasm-i8080/src/machine.rs
//
// Makabaka1880, 2026. All rights reserved.

/// A lightweight snapshot of the machine state for wasm export.
#[derive(Serialize)]
pub struct StateView {
    pub regs: [u8; 8],
    pub pc: u16,
    pub sp: u16,
    pub is_halted: bool,
    pub int_enable: bool,
}

pub struct I8080 {
    pub regs: [u8; 8],
    pub pc: u16,
    pub sp: u16,
    pub mem: [u8; 0x10000],
    pub ports: [u8; 256],
    pub int_enable: bool,
}

pub(crate) const B: u8 = 0;
pub(crate) const C: u8 = 1;
pub(crate) const D: u8 = 2;
pub(crate) const E: u8 = 3;
pub(crate) const H: u8 = 4;
pub(crate) const L: u8 = 5;
pub(crate) const F: u8 = 6;
pub(crate) const A: u8 = 7;

pub(crate) const BC: u8 = 0;
pub(crate) const DE: u8 = 1;
pub(crate) const HL: u8 = 2;
pub(crate) const SP: u8 = 3;
pub(crate) const PSW: u8 = 3; // same encoding as SP, different semantics for push/pop

impl I8080 {
    pub fn new() -> Self {
        I8080 {
            regs: [0; 8],
            pc: 0,
            sp: 0,
            mem: [0; 0x10000],
            ports: [0; 256],
            int_enable: false,
        }
    }

    pub fn state(&self, halted: bool) -> StateView {
        StateView {
            regs: self.regs,
            pc: self.pc,
            sp: self.sp,
            is_halted: halted,
            int_enable: self.int_enable,
        }
    }

    /// Return the assembly mnemonic at `addr` as an S-expression string, e.g. `(mov A B)`.
    pub fn disasm_at(&self, addr: u16) -> String {
        let pc = addr as usize;
        if pc + 3 > 0x10000 {
            return "(?)".into();
        }
        let op = self.mem[pc];
        let nxt = |n: usize| self.mem[pc + n];
        let imm8 = |n: usize| format!("{:02X}h", nxt(n));
        let imm16 = |n: usize| {
            let lo = nxt(n) as u16;
            let hi = nxt(n + 1) as u16;
            format!("{:04X}h", lo | (hi << 8))
        };
        let reg = |code: u8| match code {
            0 => "B",
            1 => "C",
            2 => "D",
            3 => "E",
            4 => "H",
            5 => "L",
            6 => "M",
            7 => "A",
            _ => "?",
        };
        let rp = |code: u8| match code & 3 {
            0 => "B",
            1 => "D",
            2 => "H",
            3 => "SP",
            _ => "?",
        };
        let cc = |code: u8| match code {
            0 => "NZ",
            1 => "Z",
            2 => "NC",
            3 => "C",
            4 => "PO",
            5 => "PE",
            6 => "P",
            7 => "M",
            _ => "?",
        };

        match op {
            0x00 => "(nop)".into(),
            0x76 => "(hlt)".into(),
            0x27 => "(daa)".into(),
            0x07 => "(rlc)".into(),
            0x0f => "(rrc)".into(),
            0x17 => "(ral)".into(),
            0x1f => "(rar)".into(),
            0x2f => "(cma)".into(),
            0x37 => "(stc)".into(),
            0x3f => "(cmc)".into(),
            0xc9 => "(ret)".into(),
            0xe9 => "(pchl)".into(),
            0xeb => "(xchg)".into(),
            0xe3 => "(xthl)".into(),
            0xf9 => "(sphl)".into(),
            0xfb => "(ei)".into(),
            0xf3 => "(di)".into(),

            0x3a => format!("(lda {})", imm16(1)),
            0x32 => format!("(sta {})", imm16(1)),
            0x2a => format!("(lhld {})", imm16(1)),
            0x22 => format!("(shld {})", imm16(1)),
            0xc3 => format!("(jmp {})", imm16(1)),
            0xcd => format!("(call {})", imm16(1)),

            0xdb => format!("(in {})", imm8(1)),
            0xd3 => format!("(out {})", imm8(1)),

            0xc6 => format!("(adi {})", imm8(1)),
            0xce => format!("(aci {})", imm8(1)),
            0xd6 => format!("(sui {})", imm8(1)),
            0xde => format!("(sbi {})", imm8(1)),
            0xe6 => format!("(ani {})", imm8(1)),
            0xee => format!("(xri {})", imm8(1)),
            0xf6 => format!("(ori {})", imm8(1)),
            0xfe => format!("(cpi {})", imm8(1)),

            _ if op & 0xc0 == 0x40 => {
                format!("(mov {} {})", reg((op >> 3) & 7), reg(op & 7))
            }

            _ if op & 0xc0 == 0x80 => {
                let src = reg(op & 7);
                let mne = match (op >> 3) & 7 {
                    0 => "add",
                    1 => "adc",
                    2 => "sub",
                    3 => "sbb",
                    4 => "ana",
                    5 => "xra",
                    6 => "ora",
                    7 => "cmp",
                    _ => "?",
                };
                format!("({} {})", mne, src)
            }

            _ if op & 0xc0 == 0x00 => match op & 0x0f {
                0x01 => format!("(lxi {} {})", rp((op >> 4) & 3), imm16(1)),
                0x02 => format!("(stax {})", rp((op >> 4) & 3)),
                0x03 => format!("(inx {})", rp((op >> 4) & 3)),
                0x09 => format!("(dad {})", rp((op >> 4) & 3)),
                0x0a => format!("(ldax {})", rp((op >> 4) & 3)),
                0x0b => format!("(dcx {})", rp((op >> 4) & 3)),
                _ => match op & 0x07 {
                    4 => format!("(inr {})", reg((op >> 3) & 7)),
                    5 => format!("(dcr {})", reg((op >> 3) & 7)),
                    6 => format!("(mvi {} {})", reg((op >> 3) & 7), imm8(1)),
                    _ => "(?)".into(),
                },
            },

            _ => match op & 0x0f {
                0x05 => format!("(push {})", rp((op >> 4) & 3)),
                0x01 => format!("(pop {})", rp((op >> 4) & 3)),
                _ => match op & 0x07 {
                    2 => format!("(jcond {} {})", cc((op >> 3) & 7), imm16(1)),
                    4 => format!("(callCond {} {})", cc((op >> 3) & 7), imm16(1)),
                    0 => format!("(retCond {})", cc((op >> 3) & 7)),
                    7 => format!("(rst {})", (op >> 3) & 7),
                    _ => "(?)".into(),
                },
            },
        }
    }

    pub(crate) fn flag_set(&self, cc: Cc) -> bool {
        let f = self.regs[F as usize];
        let s = (f >> 7) & 1;
        let z = (f >> 6) & 1;
        let p = (f >> 2) & 1;
        let cy = f & 1;
        match cc {
            Cc::NZ => z == 0,
            Cc::Z => z == 1,
            Cc::NC => cy == 0,
            Cc::C => cy == 1,
            Cc::PO => p == 0,
            Cc::PE => p == 1,
            Cc::P => s == 0,
            Cc::M => s == 1,
        }
    }
    pub(crate) fn get(&self, code: u8) -> u8 {
        let idx = (code & 0x7) as usize;
        if idx == 6 {
            let addr = ((self.regs[H as usize] as u16) << 8) | self.regs[L as usize] as u16;
            self.mem[addr as usize]
        } else {
            self.regs[idx]
        }
    }

    pub(crate) fn set(&mut self, code: u8, val: u8) {
        let idx = (code & 0x7) as usize;
        if idx == 6 {
            let addr = ((self.regs[H as usize] as u16) << 8) | self.regs[L as usize] as u16;
            self.mem[addr as usize] = val;
        } else {
            self.regs[idx] = val;
        }
    }

    pub(crate) fn get_pair(&self, rp: u8) -> u16 {
        match rp & 0x3 {
            0 => ((self.regs[B as usize] as u16) << 8) | self.regs[C as usize] as u16,
            1 => ((self.regs[D as usize] as u16) << 8) | self.regs[E as usize] as u16,
            2 => ((self.regs[H as usize] as u16) << 8) | self.regs[L as usize] as u16,
            3 => self.sp,
            _ => unreachable!(),
        }
    }

    pub(crate) fn set_pair(&mut self, rp: u8, val: u16) {
        match rp & 0x3 {
            0 => {
                self.regs[B as usize] = (val >> 8) as u8;
                self.regs[C as usize] = val as u8
            }
            1 => {
                self.regs[D as usize] = (val >> 8) as u8;
                self.regs[E as usize] = val as u8
            }
            2 => {
                self.regs[H as usize] = (val >> 8) as u8;
                self.regs[L as usize] = val as u8
            }
            3 => self.sp = val,
            _ => unreachable!(),
        }
    }

    pub(crate) fn push_word(&mut self, val: u16) {
        self.sp = self.sp.wrapping_sub(1);
        self.mem[self.sp as usize] = (val >> 8) as u8;
        self.sp = self.sp.wrapping_sub(1);
        self.mem[self.sp as usize] = val as u8;
    }

    pub(crate) fn pop_word(&mut self) -> u16 {
        let lo = self.mem[self.sp as usize] as u16;
        self.sp = self.sp.wrapping_add(1);
        let hi = self.mem[self.sp as usize] as u16;
        self.sp = self.sp.wrapping_add(1);
        hi << 8 | lo
    }

    // Note that when passing 0b11 it pushes the PSW instead of SP
    pub(crate) fn push_rp(&mut self, rp: u8) {
        let val = if rp & 0x3 == 3 {
            ((self.get(A) as u16) << 8) | self.regs[F as usize] as u16
        } else {
            self.get_pair(rp)
        };
        self.push_word(val);
    }

    pub(crate) fn pop_rp(&mut self, rp: u8) {
        let val = self.pop_word();
        if rp & 0x3 == 3 {
            self.set(A, (val >> 8) as u8);
            self.regs[F as usize] = val as u8;
        } else {
            self.set_pair(rp, val);
        }
    }

    pub(crate) fn decode(&mut self) -> Option<Instr> {
        let pc = self.pc as usize;
        let op = self.mem[pc];

        let instr = match op {
            0x00 => Instr::Nop,
            0x76 => Instr::Hlt,
            0x27 => Instr::Daa,

            0x07 => Instr::Rlc,
            0x0f => Instr::Rrc,
            0x17 => Instr::Ral,
            0x1f => Instr::Rar,
            0x2f => Instr::Cma,
            0x37 => Instr::Stc,
            0x3f => Instr::Cmc,

            0x3a => {
                let lo = self.mem[pc + 1] as u16;
                let hi = self.mem[pc + 2] as u16;
                Instr::Lda {
                    addr: lo | (hi << 8),
                }
            }
            0x32 => {
                let lo = self.mem[pc + 1] as u16;
                let hi = self.mem[pc + 2] as u16;
                Instr::Sta {
                    addr: lo | (hi << 8),
                }
            }
            0x2a => {
                let lo = self.mem[pc + 1] as u16;
                let hi = self.mem[pc + 2] as u16;
                Instr::Lhld {
                    addr: lo | (hi << 8),
                }
            }
            0x22 => {
                let lo = self.mem[pc + 1] as u16;
                let hi = self.mem[pc + 2] as u16;
                Instr::Shld {
                    addr: lo | (hi << 8),
                }
            }

            0xc3 => {
                let lo = self.mem[pc + 1] as u16;
                let hi = self.mem[pc + 2] as u16;
                Instr::Jmp {
                    addr: lo | (hi << 8),
                }
            }
            0xcd => {
                let lo = self.mem[pc + 1] as u16;
                let hi = self.mem[pc + 2] as u16;
                Instr::Call {
                    addr: lo | (hi << 8),
                }
            }
            0xc9 => Instr::Ret,
            0xe9 => Instr::Pchl,
            0xeb => Instr::Xchg,
            0xe3 => Instr::Xthl,
            0xf9 => Instr::Sphl,
            0xfb => Instr::Ei,
            0xf3 => Instr::Di,

            0xdb => Instr::In {
                port: self.mem[pc + 1],
            },
            0xd3 => Instr::Out {
                port: self.mem[pc + 1],
            },

            0xc6 => Instr::Adi {
                imm: self.mem[pc + 1],
            },
            0xce => Instr::Aci {
                imm: self.mem[pc + 1],
            },
            0xd6 => Instr::Sui {
                imm: self.mem[pc + 1],
            },
            0xde => Instr::Sbi {
                imm: self.mem[pc + 1],
            },
            0xe6 => Instr::Ani {
                imm: self.mem[pc + 1],
            },
            0xee => Instr::Xri {
                imm: self.mem[pc + 1],
            },
            0xf6 => Instr::Ori {
                imm: self.mem[pc + 1],
            },
            0xfe => Instr::Cpi {
                imm: self.mem[pc + 1],
            },

            op if op & 0xc0 == 0x40 => Instr::Mov {
                dst: (op >> 3) & 0x7,
                src: op & 0x7,
            },

            op if op & 0xc0 == 0x80 => {
                let src = op & 0x7;
                match (op >> 3) & 0x7 {
                    0 => Instr::Add { src },
                    1 => Instr::Adc { src },
                    2 => Instr::Sub { src },
                    3 => Instr::Sbb { src },
                    4 => Instr::Ana { src },
                    5 => Instr::Xra { src },
                    6 => Instr::Ora { src },
                    7 => Instr::Cmp { src },
                    _ => unreachable!(),
                }
            }

            op if op & 0xc0 == 0x00 => match op & 0x0f {
                0x01 => {
                    let lo = self.mem[pc + 1] as u16;
                    let hi = self.mem[pc + 2] as u16;
                    Instr::Lxi {
                        rp: (op >> 4) & 0x3,
                        imm: lo | (hi << 8),
                    }
                }
                0x02 => Instr::Stax {
                    rp: (op >> 4) & 0x3,
                },
                0x03 => Instr::Inx {
                    rp: (op >> 4) & 0x3,
                },
                0x09 => Instr::Dad {
                    rp: (op >> 4) & 0x3,
                },
                0x0a => Instr::Ldax {
                    rp: (op >> 4) & 0x3,
                },
                0x0b => Instr::Dcx {
                    rp: (op >> 4) & 0x3,
                },
                _ => match op & 0x07 {
                    4 => Instr::Inr {
                        dst: (op >> 3) & 0x7,
                    },
                    5 => Instr::Dcr {
                        dst: (op >> 3) & 0x7,
                    },
                    6 => Instr::Mvi {
                        dst: (op >> 3) & 0x7,
                        imm: self.mem[pc + 1],
                    },
                    _ => return None,
                },
            },

            _ => match op & 0x0f {
                0x05 => Instr::Push {
                    rp: (op >> 4) & 0x3,
                },
                0x01 => Instr::Pop {
                    rp: (op >> 4) & 0x3,
                },
                _ => match op & 0x07 {
                    2 => {
                        let lo = self.mem[pc + 1] as u16;
                        let hi = self.mem[pc + 2] as u16;
                        Instr::Jcond {
                            cc: Cc::from_bits((op >> 3) & 0x7),
                            addr: lo | (hi << 8),
                        }
                    }
                    4 => {
                        let lo = self.mem[pc + 1] as u16;
                        let hi = self.mem[pc + 2] as u16;
                        Instr::CallCond {
                            cc: Cc::from_bits((op >> 3) & 0x7),
                            addr: lo | (hi << 8),
                        }
                    }
                    0 => Instr::RetCond {
                        cc: Cc::from_bits((op >> 3) & 0x7),
                    },
                    7 => Instr::Rst { n: (op >> 3) & 0x7 },
                    _ => return None,
                },
            },
        };

        self.pc += instr.byte_size() as u16;
        Some(instr)
    }

    pub fn step(&mut self) -> Result<bool, String> {
        let Some(instr) = self.decode() else {
            self.pc += 1;
            return Ok(false);
        };

        match instr {
            Mov { dst, src } => {
                let val = self.get(src);
                self.set(dst, val);
            }
            Mvi { dst, imm } => {
                self.set(dst, imm);
            }
            Lxi { rp, imm } => {
                self.set_pair(rp, imm);
            }
            Lda { addr } => {
                self.set(A, self.mem[addr as usize]);
            }
            Sta { addr } => {
                self.mem[addr as usize] = self.get(A);
            }
            Lhld { addr } => {
                self.set(L, self.mem[addr as usize]);
                self.set(H, self.mem[addr.wrapping_add(1) as usize]);
            }
            Shld { addr } => {
                self.mem[addr as usize] = self.get(L);
                self.mem[addr.wrapping_add(1) as usize] = self.get(H)
            }
            Ldax { rp } => self.set(A, self.mem[self.get_pair(rp) as usize]),
            Stax { rp } => self.mem[self.get_pair(rp) as usize] = self.get(A),
            Xchg => {
                let de = self.get_pair(DE);
                let hl = self.get_pair(HL);
                self.set_pair(DE, hl);
                self.set_pair(HL, de);
            }
            Add { src } => {
                let a = self.get(A);
                let val = self.get(src);
                let (result, carry) = a.overflowing_add(val);
                let aux_carry = (a & 0x0F) + (val & 0x0F) > 0x0F;
                let s = (result & 0x80) != 0;
                let z = result == 0;
                let p = result.count_ones() & 0x01 == 0;
                self.set(A, result);
                self.regs[F as usize] = ((s as u8) << 7)
                    | ((z as u8) << 6)
                    | ((aux_carry as u8) << 4)
                    | ((p as u8) << 2)
                    | (carry as u8);
            }
            Adi { imm } => {
                let a = self.get(A);
                let (res, carry) = a.overflowing_add(imm);
                let aux_carry = (a & 0x0F) + (imm & 0x0F) > 0x0F;
                let s = (res & 0x80) != 0;
                let z = res == 0;
                let p = res.count_ones() & 0x01 == 0;
                self.set(A, res);
                self.regs[F as usize] = ((s as u8) << 7)
                    | ((z as u8) << 6)
                    | ((aux_carry as u8) << 4)
                    | ((p as u8) << 2)
                    | (carry as u8);
            }
            Adc { src } => {
                let a = self.get(A);
                let dat = self.get(src);
                let cin = self.regs[F as usize] & 0x01;
                let (res, carry1) = a.overflowing_add(dat);
                let (res, carry2) = res.overflowing_add(cin);
                let carry = carry1 || carry2;
                let aux_carry = (a & 0x0F) + (dat & 0x0F) + cin as u8 > 0x0F;
                let s = (res & 0x80) != 0;
                let z = res == 0;
                let p = res.count_ones() & 0x01 == 0;
                self.set(A, res);
                self.regs[F as usize] = ((s as u8) << 7)
                    | ((z as u8) << 6)
                    | ((aux_carry as u8) << 4)
                    | ((p as u8) << 2)
                    | (carry as u8);
            }
            Aci { imm } => {
                let a = self.get(A);
                let cin = self.regs[F as usize] & 0x01;
                let (res, carry1) = a.overflowing_add(imm);
                let (res, carry2) = res.overflowing_add(cin);
                let carry = carry1 || carry2;
                let aux_carry = (a & 0x0F) + (imm & 0x0F) + cin as u8 > 0x0F;
                let s = (res & 0x80) != 0;
                let z = res == 0;
                let p = res.count_ones() & 0x01 == 0;
                self.set(A, res);
                self.regs[F as usize] = ((s as u8) << 7)
                    | ((z as u8) << 6)
                    | ((aux_carry as u8) << 4)
                    | ((p as u8) << 2)
                    | (carry as u8);
            }
            Sub { src } => {
                let a = self.get(A);
                let val = self.get(src);
                let (res, borrow) = a.overflowing_sub(val);
                let aux_carry = (a & 0x0F).overflowing_sub(val & 0x0F).1;
                let s = (res & 0x80) != 0;
                let z = res == 0;
                let p = res.count_ones() & 0x01 == 0;
                self.set(A, res);
                self.regs[F as usize] = ((s as u8) << 7)
                    | ((z as u8) << 6)
                    | ((aux_carry as u8) << 4)
                    | ((p as u8) << 2)
                    | (borrow as u8);
            }
            Sui { imm } => {
                let a = self.get(A);
                let (res, borrow) = a.overflowing_sub(imm);
                let aux_carry = (a & 0x0F).overflowing_sub(imm & 0x0F).1;
                let s = (res & 0x80) != 0;
                let z = res == 0;
                let p = res.count_ones() & 0x01 == 0;
                self.set(A, res);
                self.regs[F as usize] = ((s as u8) << 7)
                    | ((z as u8) << 6)
                    | ((aux_carry as u8) << 4)
                    | ((p as u8) << 2)
                    | (borrow as u8);
            }
            Sbb { src } => {
                let a = self.get(A);
                let val = self.get(src);
                let cin = self.regs[F as usize] & 0x01;
                let (inter, borrow1) = a.overflowing_sub(val);
                let (res, borrow2) = inter.overflowing_sub(cin);
                let borrow = borrow1 || borrow2;
                let aux_carry = (a & 0x0F).overflowing_sub(val & 0x0F).1
                    || (inter & 0x0F).overflowing_sub(cin).1;
                let s = (res & 0x80) != 0;
                let z = res == 0;
                let p = res.count_ones() & 0x01 == 0;
                self.set(A, res);
                self.regs[F as usize] = ((s as u8) << 7)
                    | ((z as u8) << 6)
                    | ((aux_carry as u8) << 4)
                    | ((p as u8) << 2)
                    | (borrow as u8);
            }
            Sbi { imm } => {
                let a = self.get(A);
                let cin = self.regs[F as usize] & 0x01;
                let (inter, borrow1) = a.overflowing_sub(imm);
                let (res, borrow2) = inter.overflowing_sub(cin);
                let borrow = borrow1 || borrow2;
                let aux_carry = (a & 0x0F).overflowing_sub(imm & 0x0F).1
                    || (inter & 0x0F).overflowing_sub(cin).1;
                let s = (res & 0x80) != 0;
                let z = res == 0;
                let p = res.count_ones() & 0x01 == 0;
                self.set(A, res);
                self.regs[F as usize] = ((s as u8) << 7)
                    | ((z as u8) << 6)
                    | ((aux_carry as u8) << 4)
                    | ((p as u8) << 2)
                    | (borrow as u8);
            }
            Inr { dst } => {
                let x = self.get(dst);
                let res = x.wrapping_add(1);
                let s = (res & 0x80) != 0;
                let z = res == 0;
                let p = res.count_ones() & 0x01 == 0;
                let aux_carry = (x & 0x0F) + 1 > 0x0F;
                self.set(dst, res);
                self.regs[F as usize] = ((s as u8) << 7)
                    | ((z as u8) << 6)
                    | ((aux_carry as u8) << 4)
                    | ((p as u8) << 2)
                    | (self.regs[F as usize] & 0x01);
            }
            Dcr { dst } => {
                let x = self.get(dst);
                let res = x.wrapping_sub(1);
                let s = (res & 0x80) != 0;
                let z = res == 0;
                let p = res.count_ones() & 0x01 == 0;
                let aux_carry = (x & 0x0F) == 0;
                self.set(dst, res);
                self.regs[F as usize] = ((s as u8) << 7)
                    | ((z as u8) << 6)
                    | ((aux_carry as u8) << 4)
                    | ((p as u8) << 2)
                    | (self.regs[F as usize] & 0x01);
            }
            Inx { rp } => {
                let val = self.get_pair(rp).wrapping_add(1);
                self.set_pair(rp, val);
            }
            Dcx { rp } => {
                let val = self.get_pair(rp).wrapping_sub(1);
                self.set_pair(rp, val);
            }
            Dad { rp } => {
                let a = self.get_pair(HL) as u32;
                let b = self.get_pair(rp) as u32;
                let sum = a + b;
                self.set_pair(HL, sum as u16);
                self.regs[F as usize] =
                    (self.regs[F as usize] & !0x01) | (((sum >> 16) as u8) & 0x01);
            }
            Daa => {
                let a = self.get(A);
                let ac = (self.regs[F as usize] >> 4) & 1;
                let cy = self.regs[F as usize] & 1;

                let mut result = a as u16;

                let new_ac = if (result & 0x0F) > 9 || ac != 0 {
                    result += 6;
                    1
                } else {
                    0
                };

                let new_cy = if result > 0x99 || cy != 0 {
                    result += 0x60;
                    1
                } else {
                    0
                };

                let res = result as u8;
                let s = (res & 0x80) != 0;
                let z = res == 0;
                let p = res.count_ones() & 0x01 == 0;

                self.set(A, res);
                self.regs[F as usize] = ((s as u8) << 7)
                    | ((z as u8) << 6)
                    | (new_ac << 4)
                    | ((p as u8) << 2)
                    | ((new_cy | ((result >> 8) as u8)) & 1);
            }
            Ana { src } => {
                let a = self.get(A);
                let val = self.get(src);
                let res = a & val;
                self.set(A, res);
                let s = (res & 0x80) != 0;
                let z = res == 0;
                let p = res.count_ones() & 0x01 == 0;
                let aux = (a >> 3 | val >> 3) & 0x01 > 0; // 🤷 i8080 does so
                self.regs[F as usize] =
                    ((s as u8) << 7) | ((z as u8) << 6) | ((aux as u8) << 4) | ((p as u8) << 2);
            }
            Ani { imm } => {
                let a = self.get(A);
                let res = a & imm;
                self.set(A, res);
                let s = (res & 0x80) != 0;
                let z = res == 0;
                let p = res.count_ones() & 0x01 == 0;
                let aux = (a >> 3 | imm >> 3) & 0x01 > 0; // again idk why
                self.regs[F as usize] =
                    ((s as u8) << 7) | ((z as u8) << 6) | ((aux as u8) << 4) | ((p as u8) << 2);
            }
            Ora { src } => {
                let a = self.get(A);
                let val = self.get(src);
                let res = a | val;
                self.set(A, res);
                let s = (res & 0x80) != 0;
                let z = res == 0;
                let p = res.count_ones() & 0x01 == 0;
                self.regs[F as usize] = ((s as u8) << 7) | ((z as u8) << 6) | ((p as u8) << 2);
            }

            Ori { imm } => {
                let a = self.get(A);
                let res = a | imm;
                self.set(A, res);
                let s = (res & 0x80) != 0;
                let z = res == 0;
                let p = res.count_ones() & 0x01 == 0;
                self.regs[F as usize] = ((s as u8) << 7) | ((z as u8) << 6) | ((p as u8) << 2);
            }

            Xra { src } => {
                let a = self.get(A);
                let val = self.get(src);
                let res = a ^ val;
                self.set(A, res);
                let s = (res & 0x80) != 0;
                let z = res == 0;
                let p = res.count_ones() & 0x01 == 0;
                self.regs[F as usize] = ((s as u8) << 7) | ((z as u8) << 6) | ((p as u8) << 2);
            }

            Xri { imm } => {
                let a = self.get(A);
                let res = a ^ imm;
                self.set(A, res);
                let s = (res & 0x80) != 0;
                let z = res == 0;
                let p = res.count_ones() & 0x01 == 0;
                self.regs[F as usize] = ((s as u8) << 7) | ((z as u8) << 6) | ((p as u8) << 2);
            }

            Cmp { src } => {
                let a = self.get(A);
                let val = self.get(src);
                let (res, borrow) = a.overflowing_sub(val);
                let s = (res & 0x80) != 0;
                let z = res == 0;
                let p = res.count_ones() & 0x01 == 0;
                let aux = (a & 0x0F).overflowing_sub(val & 0x0F).1;
                self.regs[F as usize] = ((s as u8) << 7)
                    | ((z as u8) << 6)
                    | ((aux as u8) << 4)
                    | ((p as u8) << 2)
                    | (borrow as u8);
            }

            Cpi { imm } => {
                let a = self.get(A);
                let (res, borrow) = a.overflowing_sub(imm);
                let s = (res & 0x80) != 0;
                let z = res == 0;
                let p = res.count_ones() & 0x01 == 0;
                let aux = (a & 0x0F).overflowing_sub(imm & 0x0F).1;
                self.regs[F as usize] = ((s as u8) << 7)
                    | ((z as u8) << 6)
                    | ((aux as u8) << 4)
                    | ((p as u8) << 2)
                    | (borrow as u8);
            }
            Rlc => {
                let a = self.get(A);
                let hsb = a >> 7;
                self.set(A, (a << 1) | hsb);
                self.regs[F as usize] = (self.regs[F as usize] & !0x01) | hsb;
            }
            Rrc => {
                let a = self.get(A);
                let lsb = a & 0x01;
                self.set(A, (a >> 1) | lsb << 7);
                self.regs[F as usize] = (self.regs[F as usize] & !0x01) | lsb;
            }
            Ral => {
                let a = self.get(A);
                let hsb = a >> 7;
                let c = self.regs[F as usize] & 0x01;
                self.set(A, (a << 1) | c);
                self.regs[F as usize] = (self.regs[F as usize] & !0x01) | hsb;
            }
            Rar => {
                let a = self.get(A);
                let lsb = a & 0x01;
                let c = self.regs[F as usize] & 0x01;
                self.set(A, (a >> 1) | c << 7);
                self.regs[F as usize] = (self.regs[F as usize] & !0x01) | lsb;
            }
            Cma => self.set(A, !self.get(A)),
            Cmc => self.regs[F as usize] ^= 0x01,
            Stc => self.regs[F as usize] |= 0x01,
            Jmp { addr } => self.pc = addr,
            Jcond { cc, addr } => {
                if self.flag_set(cc) {
                    self.pc = addr
                }
            }
            Call { addr } => {
                self.push_word(self.pc);
                self.pc = addr;
            }
            CallCond { cc, addr } => {
                if self.flag_set(cc) {
                    self.push_word(self.pc);
                    self.pc = addr;
                }
            }
            Ret => {
                self.pc = self.pop_word();
            }
            RetCond { cc } => {
                if self.flag_set(cc) {
                    self.pc = self.pop_word();
                }
            }
            Rst { n } => {
                self.push_word(self.pc);
                self.pc = (n as u16) * 8;
            }
            Pchl => self.pc = self.get_pair(HL),
            Push { rp } => self.push_rp(rp),
            Pop { rp } => self.pop_rp(rp),
            Xthl => {
                let lo = self.mem[self.sp as usize] as u16;
                let hi = self.mem[(self.sp.wrapping_add(1)) as usize] as u16;
                let sp_val = hi << 8 | lo;
                self.mem[self.sp as usize] = self.regs[L as usize];
                self.mem[(self.sp.wrapping_add(1)) as usize] = self.regs[H as usize];
                self.set_pair(HL, sp_val);
            }
            Sphl => self.sp = self.get_pair(HL),
            In { port } => self.set(A, self.ports[port as usize]),
            Out { port } => self.ports[port as usize] = self.get(A),
            Hlt => return Ok(true),
            Nop => {}
            Ei => self.int_enable = true,
            Di => self.int_enable = false,
        }
        Ok(false)
    }
}
