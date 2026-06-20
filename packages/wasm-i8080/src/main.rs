// Created by Sean L. on Jun. 20.
// Last Updated by Sean L. on Jun. 20.
//
// sewing-box
// packages/wasm-i8080/src/main.rs
//
// Makabaka1880, 2026. All rights reserved.

// First of all this main.rs is just a debugger I let claude code write for convenience. I have absolutely no guarantee for the correctness or usablilty of anything in main.rs. 

use std::io::{self, BufRead, Write};
use wasm_i8080::machine::I8080;
use wasm_i8080::parser::parse;

const R: [&str; 8] = ["B", "C", "D", "E", "H", "L", "F", "A"];

struct PortEvent {
    step: u64,
    pc: u16,
    addr: u8,
    val: u8,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("usage: {} <source-file> <pc> <sp>", args[0]);
        eprintln!("  e.g. {} program.asm 0100h F000h", args[0]);
        std::process::exit(1);
    }

    let src = match std::fs::read_to_string(&args[1]) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("read {}: {}", args[1], e);
            std::process::exit(1);
        }
    };

    let pc = parse_hex(&args[2]);
    let sp = parse_hex(&args[3]);
    let io_bus = vec![0u8; 256];

    let blocks = match parse(&src) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("parse error: {}", e);
            std::process::exit(1);
        }
    };

    let mut machine = match I8080::assemble(blocks, io_bus, pc, sp) {
        Ok(m) => m,
        Err(e) => {
            eprintln!("assemble error: {}", e);
            std::process::exit(1);
        }
    };

    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();

    let mut port_history: Vec<PortEvent> = Vec::new();
    let mut step_count: u64 = 0;

    println!(
        "assembled. pc={:04X} sp={:04X}. commands: s, r, p, h, q, d, m [addr]",
        pc, sp
    );

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let line = match lines.next() {
            Some(Ok(l)) => l,
            _ => break,
        };
        let cmd = line.trim();

        match cmd {
            "" => {}
            "q" | "quit" => break,
            "d" | "dump" => dump_regs(&machine),
            "p" | "ports" => dump_ports(&machine),
            "h" | "hist" | "history" => dump_port_history(&port_history),
            "s" | "step" => {
                let is_out = machine.disasm_at(machine.pc).starts_with("(out");
                let prev_ports = if is_out { machine.ports } else { [0; 256] };
                let prev_pc = machine.pc;
                match machine.step() {
                    Ok(true) => {
                        step_count += 1;
                        if is_out {
                            record_ports(&mut port_history, step_count, prev_pc, &prev_ports, &machine);
                        }
                        println!("  [HLT]");
                        dump_regs(&machine);
                    }
                    Ok(false) => {
                        step_count += 1;
                        if is_out {
                            record_ports(&mut port_history, step_count, prev_pc, &prev_ports, &machine);
                        }
                        dump_regs(&machine);
                        println!("  {}", machine.disasm_at(machine.pc));
                    }
                    Err(e) => eprintln!("  error: {}", e),
                }
            }
            "r" | "run" => {
                let mut count: u64 = 0;
                loop {
                    let is_out = machine.disasm_at(machine.pc).starts_with("(out");
                    let prev_ports = if is_out { machine.ports } else { [0; 256] };
                    let prev_pc = machine.pc;
                    match machine.step() {
                        Ok(true) => {
                            count += 1;
                            if is_out {
                                record_ports(&mut port_history, step_count + count, prev_pc, &prev_ports, &machine);
                            }
                            step_count += count;
                            println!("  [HLT after {} steps]", count);
                            dump_regs(&machine);
                            break;
                        }
                        Ok(false) => {
                            count += 1;
                            if is_out {
                                record_ports(&mut port_history, step_count + count, prev_pc, &prev_ports, &machine);
                            }
                        }
                        Err(e) => {
                            step_count += count;
                            eprintln!("error at step {}: {}", count + 1, e);
                            break;
                        }
                    }
                }
            }
            _ if cmd.starts_with("m ") || cmd.starts_with("mem ") => {
                let addr_part = cmd.split_whitespace().nth(1).unwrap_or("0");
                let addr = parse_hex(addr_part) & 0xFFF0; // align to 16
                dump_memory(&machine, addr, 8);
            }
            _ => {
                // try as hex address for memory dump
                let addr = parse_hex(cmd) & 0xFFF0;
                dump_memory(&machine, addr, 8);
            }
        }
    }

    // final state
    println!();
    dump_regs(&machine);
    println!();
    dump_memory(&machine, machine.pc & 0xFFF0, 4);
}

fn parse_hex(s: &str) -> u16 {
    let s = s.trim_end_matches(|c| c == 'h' || c == 'H');
    u16::from_str_radix(s, 16).unwrap_or_else(|_| {
        eprintln!("invalid hex: {}", s);
        std::process::exit(1);
    })
}

fn dump_regs(m: &I8080) {
    print!("A={:02X} ", m.regs[7]);
    for i in 0..6 {
        print!("{}= {:02X} ", R[i], m.regs[i]);
    }
    let f = m.regs[6];
    println!(
        "F={:02X} [{}]  PC={:04X}  SP={:04X}",
        f,
        flags_str(f),
        m.pc,
        m.sp,
    );
}

fn flags_str(f: u8) -> String {
    let s = if f & 0x80 != 0 { "S" } else { "-" };
    let z = if f & 0x40 != 0 { "Z" } else { "-" };
    let ac = if f & 0x10 != 0 { "A" } else { "-" };
    let p = if f & 0x04 != 0 { "P" } else { "-" };
    let c = if f & 0x01 != 0 { "C" } else { "-" };
    format!("{}{}{}{}{}", s, z, ac, p, c)
}

fn record_ports(
    history: &mut Vec<PortEvent>,
    step: u64,
    pc: u16,
    prev: &[u8; 256],
    m: &I8080,
) {
    for addr in 0..256u16 {
        let a = addr as u8;
        let port = a as usize;
        if m.ports[port] != prev[port] {
            let ch = if (0x20..=0x7E).contains(&m.ports[port]) {
                m.ports[port] as char
            } else {
                '·'
            };
            println!(
                "  OUT {:02X} -> {:02X} {}  (step {})",
                a, m.ports[port], ch, step
            );
            history.push(PortEvent {
                step,
                pc,
                addr: a,
                val: m.ports[port],
            });
        }
    }
}

fn dump_port_history(history: &[PortEvent]) {
    if history.is_empty() {
        println!("  (no output yet)");
        return;
    }
    println!("  {:>6} {:>5} {:>5} {:>4} CHAR", "step", "PC", "PORT", "VAL");
    println!("  {} {} {} {} ----", "-----", "-----", "-----", "----");
    for e in history {
        let ch = if (0x20..=0x7E).contains(&e.val) {
            e.val as char
        } else {
            '·'
        };
        println!(
            "  {:>6} {:04X}  {:02X}   {:02X}  {}",
            e.step, e.pc, e.addr, e.val, ch
        );
    }
}

fn dump_ports(m: &I8080) {
    let mut shown = 0;
    for (addr, &val) in m.ports.iter().enumerate() {
        if val != 0 {
            let ch = if (0x20..=0x7E).contains(&val) {
                val as char
            } else {
                '·'
            };
            println!("  {:02X}: {:02X} {}", addr, val, ch);
            shown += 1;
        }
    }
    if shown == 0 {
        println!("  (all zero)");
    }
}

fn dump_memory(m: &I8080, start: u16, rows: u16) {
    for row in 0..rows {
        let addr = start + row * 16;
        print!("{:04X} │", addr);
        for col in 0..16 {
            let a = addr + col;
            let byte = m.mem[a as usize];
            if a == m.pc {
                print!("\x1b[7m{:02X}\x1b[0m ", byte);
            } else if a == m.sp {
                print!("\x1b[4m{:02X}\x1b[0m ", byte);
            } else {
                print!("{:02X} ", byte);
            }
        }
        print!("│");
        for col in 0..16 {
            let byte = m.mem[(addr + col) as usize];
            let ch = if (0x20..=0x7E).contains(&byte) {
                byte as char
            } else {
                '.'
            };
            print!("{}", ch);
        }
        println!("│");
    }
}
