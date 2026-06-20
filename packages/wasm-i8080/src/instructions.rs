// Created by Sean L. on Jun. 20.
// Last Updated by Sean L. on Jun. 20.
//
// sewing-box
// packages/wasm-i8080/src/instructions.rs
//
// Makabaka1880, 2026. All rights reserved.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Cc {
    NZ = 0,
    Z = 1,
    NC = 2,
    C = 3,
    PO = 4,
    PE = 5,
    P = 6,
    M = 7,
}

impl Cc {
    pub fn from_bits(bits: u8) -> Self {
        match bits & 0x7 {
            0 => Cc::NZ,
            1 => Cc::Z,
            2 => Cc::NC,
            3 => Cc::C,
            4 => Cc::PO,
            5 => Cc::PE,
            6 => Cc::P,
            7 => Cc::M,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Instr {
    Mov { dst: u8, src: u8 },
    Mvi { dst: u8, imm: u8 },
    Lxi { rp: u8, imm: u16 },
    Lda { addr: u16 },
    Sta { addr: u16 },
    Lhld { addr: u16 },
    Shld { addr: u16 },
    Ldax { rp: u8 },
    Stax { rp: u8 },
    Xchg,

    Add { src: u8 },
    Adc { src: u8 },
    Sub { src: u8 },
    Sbb { src: u8 },

    Adi { imm: u8 },
    Aci { imm: u8 },
    Sui { imm: u8 },
    Sbi { imm: u8 },

    Inr { dst: u8 },
    Dcr { dst: u8 },
    Inx { rp: u8 },
    Dcx { rp: u8 },

    Dad { rp: u8 },
    Daa,

    Ana { src: u8 },
    Xra { src: u8 },
    Ora { src: u8 },
    Cmp { src: u8 },

    Ani { imm: u8 },
    Xri { imm: u8 },
    Ori { imm: u8 },
    Cpi { imm: u8 },

    Rlc,
    Rrc,
    Ral,
    Rar,
    Cma,
    Cmc,
    Stc,

    Jmp { addr: u16 },
    Jcond { cc: Cc, addr: u16 },
    Call { addr: u16 },
    CallCond { cc: Cc, addr: u16 },
    Ret,
    RetCond { cc: Cc },
    Rst { n: u8 },
    Pchl,

    Push { rp: u8 },
    Pop { rp: u8 },
    Xthl,
    Sphl,

    In { port: u8 },
    Out { port: u8 },

    Ei,
    Di,
    Hlt,
    Nop,
}

impl Instr {
    pub fn byte_size(self) -> u8 {
        match self {
            Instr::Mov { .. }
            | Instr::Add { .. }
            | Instr::Adc { .. }
            | Instr::Sub { .. }
            | Instr::Sbb { .. }
            | Instr::Ana { .. }
            | Instr::Xra { .. }
            | Instr::Ora { .. }
            | Instr::Cmp { .. }
            | Instr::Inr { .. }
            | Instr::Dcr { .. }
            | Instr::Inx { .. }
            | Instr::Dcx { .. }
            | Instr::Dad { .. }
            | Instr::Ldax { .. }
            | Instr::Stax { .. }
            | Instr::Rlc
            | Instr::Rrc
            | Instr::Ral
            | Instr::Rar
            | Instr::Cma
            | Instr::Cmc
            | Instr::Stc
            | Instr::Daa
            | Instr::Xchg
            | Instr::Pchl
            | Instr::Xthl
            | Instr::Sphl
            | Instr::Ret
            | Instr::RetCond { .. }
            | Instr::Push { .. }
            | Instr::Pop { .. }
            | Instr::Ei
            | Instr::Di
            | Instr::Hlt
            | Instr::Nop
            | Instr::Rst { .. } => 1,

            Instr::Mvi { .. }
            | Instr::Adi { .. }
            | Instr::Aci { .. }
            | Instr::Sui { .. }
            | Instr::Sbi { .. }
            | Instr::Ani { .. }
            | Instr::Xri { .. }
            | Instr::Ori { .. }
            | Instr::Cpi { .. }
            | Instr::In { .. }
            | Instr::Out { .. } => 2,

            Instr::Lxi { .. }
            | Instr::Lda { .. }
            | Instr::Sta { .. }
            | Instr::Lhld { .. }
            | Instr::Shld { .. }
            | Instr::Jmp { .. }
            | Instr::Jcond { .. }
            | Instr::Call { .. }
            | Instr::CallCond { .. } => 3,
        }
    }
}
