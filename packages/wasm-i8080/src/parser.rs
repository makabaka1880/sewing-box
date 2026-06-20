// Created by Sean L. on Jun. 20.
// Last Updated by Claude Code on Jun. 20.
//
// sewing-box
// packages/wasm-i8080/src/parser.rs
//
// Makabaka1880, 2026. All rights reserved.

use crate::instructions::{Cc, Instr};
use sexp::{Atom, Sexp};

pub struct Block {
    pub anchor: u16,
    pub content: Vec<Data>,
}

pub enum Data {
    Code(Instr),
    Bytes(Vec<u8>),
}

pub fn parse(src: &str) -> Result<Vec<Block>, String> {
    let stripped = src
        .lines()
        .map(|line| {
            if let Some(pos) = line.find(';') {
                &line[..pos]
            } else {
                line
            }
        })
        .collect::<Vec<&str>>()
        .join("\n");
    let wrapped = format!("({})", stripped);
    let sexp = sexp::parse(&wrapped).map_err(|e| e.to_string())?;
    match sexp {
        Sexp::List(items) => items.into_iter().map(parse_block).collect(),
        _ => Err("Expected a sequence of blocks".to_string()),
    }
}

fn parse_block(sexp: Sexp) -> Result<Block, String> {
    let items = match sexp {
        Sexp::List(items) => items,
        _ => return Err("Block must be a list".to_string()),
    };

    if items.is_empty() {
        return Err("Empty block".to_string());
    }

    let keyword = expect_symbol(&items[0])?;
    if keyword != "block" {
        return Err(format!("Expected 'block', got '{}'", keyword));
    }
    if items.len() < 2 {
        return Err("Block missing anchor".to_string());
    }

    let anchor = parse_anchor(&items[1])?;

    let mut content = Vec::new();
    for item in &items[2..] {
        content.push(parse_item(item)?);
    }

    Ok(Block { anchor, content })
}

fn parse_item(item: &Sexp) -> Result<Data, String> {
    let items = match item {
        Sexp::List(items) => items,
        _ => return Err("Block items must be lists".to_string()),
    };
    if items.is_empty() {
        return Err("Empty list in block".to_string());
    }
    match &items[0] {
        Sexp::Atom(Atom::S(s)) if s == "db" => parse_db(items),
        _ => parse_instr(items).map(Data::Code),
    }
}

fn parse_anchor(sexp: &Sexp) -> Result<u16, String> {
    match sexp {
        Sexp::Atom(Atom::I(i)) => {
            if *i >= 0 && *i <= 0xFFFF {
                Ok(*i as u16)
            } else {
                Err(format!("Address out of range: {}", i))
            }
        }
        Sexp::Atom(Atom::S(s)) => {
            if let Some(hex) = s.strip_suffix('h').or_else(|| s.strip_suffix('H')) {
                if hex.is_empty() || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
                    return Err(format!("Invalid hex address: '{}'", s));
                }
                return u16::from_str_radix(hex, 16)
                    .map_err(|e| format!("Invalid hex address '{}': {}", s, e));
            }

            if let Ok(n) = s.parse::<u16>() {
                return Ok(n);
            }
            Err(format!(
                "Anchor must be an absolute address, got label: '{}'",
                s
            ))
        }
        Sexp::Atom(Atom::F(_)) => Err("Anchor must be an integer address, got float".to_string()),
        Sexp::List(_) => Err("Anchor must be an atom".to_string()),
    }
}

fn parse_db(items: &[Sexp]) -> Result<Data, String> {
    let mut bytes = Vec::new();
    for item in &items[1..] {
        bytes.push(parse_byte(item)?);
    }
    Ok(Data::Bytes(bytes))
}

fn parse_byte(sexp: &Sexp) -> Result<u8, String> {
    match sexp {
        Sexp::Atom(Atom::I(i)) => {
            if *i >= 0 && *i <= 0xFF {
                Ok(*i as u8)
            } else {
                Err(format!("Byte value out of range: {}", i))
            }
        }
        Sexp::Atom(Atom::S(s)) => {
            if s.starts_with('\'') && s.ends_with('\'') && s.len() == 3 {
                return Ok(s.as_bytes()[1]);
            }
            parse_hex_or_decimal_byte(s)
        }
        Sexp::Atom(Atom::F(_)) => Err("Expected integer byte value, got float".to_string()),
        Sexp::List(_) => Err("Expected byte value, got list".to_string()),
    }
}

fn parse_hex_or_decimal_byte(s: &str) -> Result<u8, String> {
    if let Some(hex) = s.strip_suffix('h').or_else(|| s.strip_suffix('H')) {
        if hex.is_empty() {
            return Err("Empty hex literal".to_string());
        }
        u8::from_str_radix(hex, 16).map_err(|e| format!("Invalid hex byte '{}': {}", s, e))
    } else {
        s.parse::<u8>()
            .map_err(|_| format!("Invalid byte value: '{}'", s))
    }
}

fn parse_word(sexp: &Sexp) -> Result<u16, String> {
    match sexp {
        Sexp::Atom(Atom::I(i)) => {
            if *i >= 0 && *i <= 0xFFFF {
                Ok(*i as u16)
            } else {
                Err(format!("Word value out of range: {}", i))
            }
        }
        Sexp::Atom(Atom::S(s)) => {
            if let Some(hex) = s.strip_suffix('h').or_else(|| s.strip_suffix('H')) {
                if hex.is_empty() {
                    return Err("Empty hex literal".to_string());
                }
                u16::from_str_radix(hex, 16).map_err(|e| format!("Invalid hex word '{}': {}", s, e))
            } else {
                s.parse::<u16>()
                    .map_err(|_| format!("Invalid word value: '{}'", s))
            }
        }
        Sexp::Atom(Atom::F(_)) => Err("Expected integer word value, got float".to_string()),
        Sexp::List(_) => Err("Expected word value, got list".to_string()),
    }
}

fn parse_reg(sexp: &Sexp) -> Result<u8, String> {
    let s = expect_symbol(sexp)?;
    match s.to_uppercase().as_str() {
        "B" => Ok(0),
        "C" => Ok(1),
        "D" => Ok(2),
        "E" => Ok(3),
        "H" => Ok(4),
        "L" => Ok(5),
        "M" => Ok(6),
        "A" => Ok(7),
        _ => Err(format!("Unknown register: '{}'", s)),
    }
}

fn parse_rp(sexp: &Sexp) -> Result<u8, String> {
    let s = expect_symbol(sexp)?;
    match s.to_uppercase().as_str() {
        "B" => Ok(0), // BC
        "D" => Ok(1), // DE
        "H" => Ok(2), // HL
        "SP" => Ok(3),
        "PSW" => Ok(3),
        _ => Err(format!("Unknown register pair: '{}'", s)),
    }
}

fn parse_cc(sexp: &Sexp) -> Result<Cc, String> {
    let s = expect_symbol(sexp)?;
    match s.to_uppercase().as_str() {
        "NZ" => Ok(Cc::NZ),
        "Z" => Ok(Cc::Z),
        "NC" => Ok(Cc::NC),
        "C" => Ok(Cc::C),
        "PO" => Ok(Cc::PO),
        "PE" => Ok(Cc::PE),
        "P" => Ok(Cc::P),
        "M" => Ok(Cc::M),
        _ => Err(format!("Unknown condition code: '{}'", s)),
    }
}

fn expect_symbol(sexp: &Sexp) -> Result<&str, String> {
    match sexp {
        Sexp::Atom(Atom::S(s)) => Ok(s.as_str()),
        Sexp::Atom(Atom::I(_)) => Err(format!("Expected identifier, got number: {}", sexp)),
        Sexp::Atom(Atom::F(_)) => Err(format!("Expected identifier, got float: {}", sexp)),
        Sexp::List(_) => Err("Expected identifier, got list".to_string()),
    }
}

fn no_args(mnemonic: &str, args: &[Sexp]) -> Result<(), String> {
    if args.is_empty() {
        Ok(())
    } else {
        Err(format!("'{}' takes no operands", mnemonic))
    }
}

fn one_arg<'a>(mnemonic: &str, args: &'a [Sexp]) -> Result<&'a Sexp, String> {
    if args.len() == 1 {
        Ok(&args[0])
    } else {
        Err(format!(
            "'{}' expects 1 operand, got {}",
            mnemonic,
            args.len()
        ))
    }
}

fn two_args<'a>(mnemonic: &str, args: &'a [Sexp]) -> Result<(&'a Sexp, &'a Sexp), String> {
    if args.len() == 2 {
        Ok((&args[0], &args[1]))
    } else {
        Err(format!(
            "'{}' expects 2 operands, got {}",
            mnemonic,
            args.len()
        ))
    }
}

fn parse_instr(items: &[Sexp]) -> Result<Instr, String> {
    let mnemonic = expect_symbol(&items[0])?;
    let args = &items[1..];

    match mnemonic {
        "nop" => {
            no_args(mnemonic, args)?;
            Ok(Instr::Nop)
        }
        "hlt" => {
            no_args(mnemonic, args)?;
            Ok(Instr::Hlt)
        }
        "ei" => {
            no_args(mnemonic, args)?;
            Ok(Instr::Ei)
        }
        "di" => {
            no_args(mnemonic, args)?;
            Ok(Instr::Di)
        }
        "rlc" => {
            no_args(mnemonic, args)?;
            Ok(Instr::Rlc)
        }
        "rrc" => {
            no_args(mnemonic, args)?;
            Ok(Instr::Rrc)
        }
        "ral" => {
            no_args(mnemonic, args)?;
            Ok(Instr::Ral)
        }
        "rar" => {
            no_args(mnemonic, args)?;
            Ok(Instr::Rar)
        }
        "cma" => {
            no_args(mnemonic, args)?;
            Ok(Instr::Cma)
        }
        "cmc" => {
            no_args(mnemonic, args)?;
            Ok(Instr::Cmc)
        }
        "stc" => {
            no_args(mnemonic, args)?;
            Ok(Instr::Stc)
        }
        "daa" => {
            no_args(mnemonic, args)?;
            Ok(Instr::Daa)
        }
        "xchg" => {
            no_args(mnemonic, args)?;
            Ok(Instr::Xchg)
        }
        "pchl" => {
            no_args(mnemonic, args)?;
            Ok(Instr::Pchl)
        }
        "xthl" => {
            no_args(mnemonic, args)?;
            Ok(Instr::Xthl)
        }
        "sphl" => {
            no_args(mnemonic, args)?;
            Ok(Instr::Sphl)
        }
        "ret" => {
            no_args(mnemonic, args)?;
            Ok(Instr::Ret)
        }

        "add" => {
            let a = one_arg(mnemonic, args)?;
            Ok(Instr::Add { src: parse_reg(a)? })
        }
        "adc" => {
            let a = one_arg(mnemonic, args)?;
            Ok(Instr::Adc { src: parse_reg(a)? })
        }
        "sub" => {
            let a = one_arg(mnemonic, args)?;
            Ok(Instr::Sub { src: parse_reg(a)? })
        }
        "sbb" => {
            let a = one_arg(mnemonic, args)?;
            Ok(Instr::Sbb { src: parse_reg(a)? })
        }
        "ana" => {
            let a = one_arg(mnemonic, args)?;
            Ok(Instr::Ana { src: parse_reg(a)? })
        }
        "xra" => {
            let a = one_arg(mnemonic, args)?;
            Ok(Instr::Xra { src: parse_reg(a)? })
        }
        "ora" => {
            let a = one_arg(mnemonic, args)?;
            Ok(Instr::Ora { src: parse_reg(a)? })
        }
        "cmp" => {
            let a = one_arg(mnemonic, args)?;
            Ok(Instr::Cmp { src: parse_reg(a)? })
        }

        "inr" => {
            let a = one_arg(mnemonic, args)?;
            Ok(Instr::Inr { dst: parse_reg(a)? })
        }
        "dcr" => {
            let a = one_arg(mnemonic, args)?;
            Ok(Instr::Dcr { dst: parse_reg(a)? })
        }

        "inx" => {
            let a = one_arg(mnemonic, args)?;
            Ok(Instr::Inx { rp: parse_rp(a)? })
        }
        "dcx" => {
            let a = one_arg(mnemonic, args)?;
            Ok(Instr::Dcx { rp: parse_rp(a)? })
        }
        "dad" => {
            let a = one_arg(mnemonic, args)?;
            Ok(Instr::Dad { rp: parse_rp(a)? })
        }
        "ldax" => {
            let a = one_arg(mnemonic, args)?;
            Ok(Instr::Ldax { rp: parse_rp(a)? })
        }
        "stax" => {
            let a = one_arg(mnemonic, args)?;
            Ok(Instr::Stax { rp: parse_rp(a)? })
        }
        "push" => {
            let a = one_arg(mnemonic, args)?;
            Ok(Instr::Push { rp: parse_rp(a)? })
        }
        "pop" => {
            let a = one_arg(mnemonic, args)?;
            Ok(Instr::Pop { rp: parse_rp(a)? })
        }

        "mov" => {
            let (dst, src) = two_args(mnemonic, args)?;
            Ok(Instr::Mov {
                dst: parse_reg(dst)?,
                src: parse_reg(src)?,
            })
        }

        "mvi" => {
            let (dst, imm) = two_args(mnemonic, args)?;
            Ok(Instr::Mvi {
                dst: parse_reg(dst)?,
                imm: parse_byte(imm)?,
            })
        }

        "adi" => {
            let a = one_arg(mnemonic, args)?;
            Ok(Instr::Adi {
                imm: parse_byte(a)?,
            })
        }
        "aci" => {
            let a = one_arg(mnemonic, args)?;
            Ok(Instr::Aci {
                imm: parse_byte(a)?,
            })
        }
        "sui" => {
            let a = one_arg(mnemonic, args)?;
            Ok(Instr::Sui {
                imm: parse_byte(a)?,
            })
        }
        "sbi" => {
            let a = one_arg(mnemonic, args)?;
            Ok(Instr::Sbi {
                imm: parse_byte(a)?,
            })
        }
        "ani" => {
            let a = one_arg(mnemonic, args)?;
            Ok(Instr::Ani {
                imm: parse_byte(a)?,
            })
        }
        "xri" => {
            let a = one_arg(mnemonic, args)?;
            Ok(Instr::Xri {
                imm: parse_byte(a)?,
            })
        }
        "ori" => {
            let a = one_arg(mnemonic, args)?;
            Ok(Instr::Ori {
                imm: parse_byte(a)?,
            })
        }
        "cpi" => {
            let a = one_arg(mnemonic, args)?;
            Ok(Instr::Cpi {
                imm: parse_byte(a)?,
            })
        }

        "in" => {
            let a = one_arg(mnemonic, args)?;
            Ok(Instr::In {
                port: parse_byte(a)?,
            })
        }
        "out" => {
            let a = one_arg(mnemonic, args)?;
            Ok(Instr::Out {
                port: parse_byte(a)?,
            })
        }

        "lxi" => {
            let (rp, imm) = two_args(mnemonic, args)?;
            Ok(Instr::Lxi {
                rp: parse_rp(rp)?,
                imm: parse_word(imm)?,
            })
        }

        "lda" => {
            let a = one_arg(mnemonic, args)?;
            Ok(Instr::Lda {
                addr: parse_word(a)?,
            })
        }
        "sta" => {
            let a = one_arg(mnemonic, args)?;
            Ok(Instr::Sta {
                addr: parse_word(a)?,
            })
        }
        "lhld" => {
            let a = one_arg(mnemonic, args)?;
            Ok(Instr::Lhld {
                addr: parse_word(a)?,
            })
        }
        "shld" => {
            let a = one_arg(mnemonic, args)?;
            Ok(Instr::Shld {
                addr: parse_word(a)?,
            })
        }

        "jmp" => {
            let a = one_arg(mnemonic, args)?;
            Ok(Instr::Jmp {
                addr: parse_word(a)?,
            })
        }
        "call" => {
            let a = one_arg(mnemonic, args)?;
            Ok(Instr::Call {
                addr: parse_word(a)?,
            })
        }

        "jcond" => {
            let (cc, addr) = two_args(mnemonic, args)?;
            Ok(Instr::Jcond {
                cc: parse_cc(cc)?,
                addr: parse_word(addr)?,
            })
        }
        "callCond" => {
            let (cc, addr) = two_args(mnemonic, args)?;
            Ok(Instr::CallCond {
                cc: parse_cc(cc)?,
                addr: parse_word(addr)?,
            })
        }
        "retCond" => {
            let a = one_arg(mnemonic, args)?;
            Ok(Instr::RetCond { cc: parse_cc(a)? })
        }

        "rst" => {
            let a = one_arg(mnemonic, args)?;
            let n = parse_byte(a)?;
            if n > 7 {
                return Err(format!("RST vector must be 0-7, got {}", n));
            }
            Ok(Instr::Rst { n })
        }

        "jnz" => {
            let a = one_arg(mnemonic, args)?;
            Ok(Instr::Jcond {
                cc: Cc::NZ,
                addr: parse_word(a)?,
            })
        }
        "jz" => {
            let a = one_arg(mnemonic, args)?;
            Ok(Instr::Jcond {
                cc: Cc::Z,
                addr: parse_word(a)?,
            })
        }
        "jnc" => {
            let a = one_arg(mnemonic, args)?;
            Ok(Instr::Jcond {
                cc: Cc::NC,
                addr: parse_word(a)?,
            })
        }
        "jc" => {
            let a = one_arg(mnemonic, args)?;
            Ok(Instr::Jcond {
                cc: Cc::C,
                addr: parse_word(a)?,
            })
        }
        "jpo" => {
            let a = one_arg(mnemonic, args)?;
            Ok(Instr::Jcond {
                cc: Cc::PO,
                addr: parse_word(a)?,
            })
        }
        "jpe" => {
            let a = one_arg(mnemonic, args)?;
            Ok(Instr::Jcond {
                cc: Cc::PE,
                addr: parse_word(a)?,
            })
        }
        "jp" => {
            let a = one_arg(mnemonic, args)?;
            Ok(Instr::Jcond {
                cc: Cc::P,
                addr: parse_word(a)?,
            })
        }
        "jm" => {
            let a = one_arg(mnemonic, args)?;
            Ok(Instr::Jcond {
                cc: Cc::M,
                addr: parse_word(a)?,
            })
        }

        "cnz" => {
            let a = one_arg(mnemonic, args)?;
            Ok(Instr::CallCond {
                cc: Cc::NZ,
                addr: parse_word(a)?,
            })
        }
        "cz" => {
            let a = one_arg(mnemonic, args)?;
            Ok(Instr::CallCond {
                cc: Cc::Z,
                addr: parse_word(a)?,
            })
        }
        "cnc" => {
            let a = one_arg(mnemonic, args)?;
            Ok(Instr::CallCond {
                cc: Cc::NC,
                addr: parse_word(a)?,
            })
        }
        "cc" => {
            let a = one_arg(mnemonic, args)?;
            Ok(Instr::CallCond {
                cc: Cc::C,
                addr: parse_word(a)?,
            })
        }
        "cpo" => {
            let a = one_arg(mnemonic, args)?;
            Ok(Instr::CallCond {
                cc: Cc::PO,
                addr: parse_word(a)?,
            })
        }
        "cpe" => {
            let a = one_arg(mnemonic, args)?;
            Ok(Instr::CallCond {
                cc: Cc::PE,
                addr: parse_word(a)?,
            })
        }
        "cp" => {
            let a = one_arg(mnemonic, args)?;
            Ok(Instr::CallCond {
                cc: Cc::P,
                addr: parse_word(a)?,
            })
        }
        "cm" => {
            let a = one_arg(mnemonic, args)?;
            Ok(Instr::CallCond {
                cc: Cc::M,
                addr: parse_word(a)?,
            })
        }

        "rnz" => {
            no_args(mnemonic, args)?;
            Ok(Instr::RetCond { cc: Cc::NZ })
        }
        "rz" => {
            no_args(mnemonic, args)?;
            Ok(Instr::RetCond { cc: Cc::Z })
        }
        "rnc" => {
            no_args(mnemonic, args)?;
            Ok(Instr::RetCond { cc: Cc::NC })
        }
        "rc" => {
            no_args(mnemonic, args)?;
            Ok(Instr::RetCond { cc: Cc::C })
        }
        "rpo" => {
            no_args(mnemonic, args)?;
            Ok(Instr::RetCond { cc: Cc::PO })
        }
        "rpe" => {
            no_args(mnemonic, args)?;
            Ok(Instr::RetCond { cc: Cc::PE })
        }
        "rp" => {
            no_args(mnemonic, args)?;
            Ok(Instr::RetCond { cc: Cc::P })
        }
        "rm" => {
            no_args(mnemonic, args)?;
            Ok(Instr::RetCond { cc: Cc::M })
        }

        _ => Err(format!("Unknown mnemonic: '{}'", mnemonic)),
    }
}
