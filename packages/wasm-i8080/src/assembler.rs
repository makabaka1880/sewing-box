// Created by Sean L. on Jun. 20.
// Last Updated by Sean L. on Jun. 20.
//
// sewing-box
// packages/wasm-i8080/src/assembler.rs
//
// Makabaka1880, 2026. All rights reserved.

use crate::instructions::Instr;
use crate::machine::I8080;
use crate::parser::Block;

impl I8080 {
    pub fn assemble(layout: Vec<Block>, io_bus: Vec<u8>, pc: u16, sp: u16) -> Result<Self, String> {
        let mut memory = [0u8; 0x10000];
        let mut occupied = [false; 0x10000];

        for block in &layout {
            let start = block.anchor;
            let end = block.content.iter().try_fold(start, |acc, item| {
                let size = match item {
                    crate::parser::Data::Code(instr) => instr.byte_size() as u16,
                    crate::parser::Data::Bytes(items) => u16::try_from(items.len())
                        .map_err(|_| format!("byte literal exceeds u16 address space: {} bytes", items.len()))?,
                };
                acc.checked_add(size)
                    .ok_or_else(|| format!("block at 0x{start:04X} overflows u16 address space"))
            })?;

            if end == 0 && start != 0 {
                return Err(format!("block at 0x{start:04X} overflows address space"));
            }
            if (start..end).any(|c| occupied[c as usize]) {
                return Err(format!("block at 0x{start:04X} overlaps existing data"));
            }
            occupied[start as usize..end as usize].fill(true);

            // emit bytes
            let mut addr = start as usize;
            for item in &block.content {
                match item {
                    crate::parser::Data::Code(instr) => {
                        let bytes = encode(*instr);
                        for b in bytes {
                            memory[addr] = b;
                            addr += 1;
                        }
                    }
                    crate::parser::Data::Bytes(items) => {
                        for &b in items {
                            memory[addr] = b;
                            addr += 1;
                        }
                    }
                }
            }
        }

        let ports: [u8; 256] = io_bus
            .try_into()
            .map_err(|v: Vec<u8>| format!("io_bus must be 256 bytes, got {}", v.len()))?;

        Ok(I8080 {
            regs: [0; 8],
            pc,
            sp,
            mem: memory,
            ports,
            int_enable: false,
        })
    }
}

/// Encode an instruction into its 8080 byte representation.
fn encode(instr: Instr) -> Vec<u8> {
    match instr {
        Instr::Nop => vec![0x00],
        Instr::Hlt => vec![0x76],
        Instr::Daa => vec![0x27],
        Instr::Rlc => vec![0x07],
        Instr::Rrc => vec![0x0f],
        Instr::Ral => vec![0x17],
        Instr::Rar => vec![0x1f],
        Instr::Cma => vec![0x2f],
        Instr::Stc => vec![0x37],
        Instr::Cmc => vec![0x3f],
        Instr::Ret => vec![0xc9],
        Instr::Pchl => vec![0xe9],
        Instr::Xchg => vec![0xeb],
        Instr::Xthl => vec![0xe3],
        Instr::Sphl => vec![0xf9],
        Instr::Ei => vec![0xfb],
        Instr::Di => vec![0xf3],

        Instr::Lda { addr } => vec![0x3a, addr as u8, (addr >> 8) as u8],
        Instr::Sta { addr } => vec![0x32, addr as u8, (addr >> 8) as u8],
        Instr::Lhld { addr } => vec![0x2a, addr as u8, (addr >> 8) as u8],
        Instr::Shld { addr } => vec![0x22, addr as u8, (addr >> 8) as u8],
        Instr::Jmp { addr } => vec![0xc3, addr as u8, (addr >> 8) as u8],
        Instr::Call { addr } => vec![0xcd, addr as u8, (addr >> 8) as u8],

        Instr::In { port } => vec![0xdb, port],
        Instr::Out { port } => vec![0xd3, port],

        Instr::Adi { imm } => vec![0xc6, imm],
        Instr::Aci { imm } => vec![0xce, imm],
        Instr::Sui { imm } => vec![0xd6, imm],
        Instr::Sbi { imm } => vec![0xde, imm],
        Instr::Ani { imm } => vec![0xe6, imm],
        Instr::Xri { imm } => vec![0xee, imm],
        Instr::Ori { imm } => vec![0xf6, imm],
        Instr::Cpi { imm } => vec![0xfe, imm],

        Instr::Mov { dst, src } => vec![0x40 | (dst & 7) << 3 | (src & 7)],

        Instr::Add { src } => vec![0x80 | (src & 7)],
        Instr::Adc { src } => vec![0x88 | (src & 7)],
        Instr::Sub { src } => vec![0x90 | (src & 7)],
        Instr::Sbb { src } => vec![0x98 | (src & 7)],
        Instr::Ana { src } => vec![0xa0 | (src & 7)],
        Instr::Xra { src } => vec![0xa8 | (src & 7)],
        Instr::Ora { src } => vec![0xb0 | (src & 7)],
        Instr::Cmp { src } => vec![0xb8 | (src & 7)],

        Instr::Mvi { dst, imm } => vec![0x06 | (dst & 7) << 3, imm],
        Instr::Inr { dst } => vec![0x04 | (dst & 7) << 3],
        Instr::Dcr { dst } => vec![0x05 | (dst & 7) << 3],

        Instr::Lxi { rp, imm } => vec![0x01 | (rp & 3) << 4, imm as u8, (imm >> 8) as u8],
        Instr::Stax { rp } => vec![0x02 | (rp & 3) << 4],
        Instr::Ldax { rp } => vec![0x0a | (rp & 3) << 4],
        Instr::Inx { rp } => vec![0x03 | (rp & 3) << 4],
        Instr::Dcx { rp } => vec![0x0b | (rp & 3) << 4],
        Instr::Dad { rp } => vec![0x09 | (rp & 3) << 4],

        Instr::Push { rp } => vec![0xc5 | (rp & 3) << 4],
        Instr::Pop { rp } => vec![0xc1 | (rp & 3) << 4],

        Instr::Jcond { cc, addr } => vec![0xc2 | (cc as u8) << 3, addr as u8, (addr >> 8) as u8],
        Instr::CallCond { cc, addr } => vec![0xc4 | (cc as u8) << 3, addr as u8, (addr >> 8) as u8],
        Instr::RetCond { cc } => vec![0xc0 | (cc as u8) << 3],
        Instr::Rst { n } => vec![0xc7 | (n & 7) << 3],
    }
}
