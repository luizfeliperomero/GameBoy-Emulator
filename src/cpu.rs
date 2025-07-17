use crate::gpu::Drawable;
use crate::memory::Memory;
use colored::Colorize;
use std::fmt;
use std::io::{self, Write};
use std::thread;
use std::time::{Duration, Instant};

#[cfg(feature = "debug")]
use prettytable::{Cell, Row, Table, format};

const FREQUENCY: u32 = 4_194_304;

#[derive(Debug, PartialEq)]
enum Instruction {
    ADC_A_n8,
    LD_H_HL,
    Call_Z_a16(bool),
    DEC_BC,
    INC_BC,
    LD_HL_E,
    NOP,
    LD_SP_n16,
    XOR_A_A,
    LD_HL_n16,
    LD_HL_A,
    LD_HL_DEC_A,
    PREFIX,
    JR_NZ_e8(bool),
    LD_C_n8,
    LD_A_n8,
    LDH_C_A,
    INC_C,
    LDH_a8_A,
    LD_B_A,
    PUSH_HL,
    LD_DE_n16,
}

struct InstructionData {
    mnemonic: &'static str,
    opcode: u8,
    cycles: u8,
}

impl Instruction {
    fn data(&self) -> InstructionData {
        match self {
            Instruction::ADC_A_n8 => InstructionData {
                mnemonic: "ADC A, n8",
                opcode: 0xCE,
                cycles: 8,
            },
            Instruction::LD_H_HL => InstructionData {
                mnemonic: "LD H, [HL]",
                opcode: 0x66,
                cycles: 8,
            },
            Instruction::Call_Z_a16(z) => InstructionData {
                mnemonic: "Call Z, a16",
                opcode: 0xCC,
                cycles: if *z { 24 } else { 12 },
            },
            Instruction::DEC_BC => InstructionData {
                mnemonic: "DEC BC",
                opcode: 0x0B,
                cycles: 8,
            },
            Instruction::INC_BC => InstructionData {
                mnemonic: "INC BC",
                opcode: 0x03,
                cycles: 8,
            },
            Instruction::LD_HL_E => InstructionData {
                mnemonic: "LD [HL], E",
                opcode: 0x73,
                cycles: 8,
            },
            Instruction::NOP => InstructionData {
                mnemonic: "NO OP",
                opcode: 0x00,
                cycles: 4,
            },
            Instruction::LD_SP_n16 => InstructionData {
                mnemonic: "LD SP, n16",
                opcode: 0x31,
                cycles: 12,
            },
            Instruction::XOR_A_A => InstructionData {
                mnemonic: "XOR A, A",
                opcode: 0xAF,
                cycles: 4,
            },
            Instruction::LD_HL_n16 => InstructionData {
                mnemonic: "LD HL, n16",
                opcode: 0x21,
                cycles: 12,
            },
            Instruction::LD_HL_DEC_A => InstructionData {
                mnemonic: "LD [HL-], A",
                opcode: 0x32,
                cycles: 8,
            },
            Instruction::PREFIX => InstructionData {
                mnemonic: "PREFIX",
                opcode: 0xCB,
                cycles: 4,
            },
            Instruction::JR_NZ_e8(z) => InstructionData {
                mnemonic: "JR NZ, e8",
                opcode: 0x20,
                cycles: if *z {12} else {8},
            },
            Instruction::LD_C_n8 => InstructionData {
                mnemonic: "LD C, n8",
                opcode: 0x0E,
                cycles: 8,
            },
            Instruction::LD_A_n8 => InstructionData {
                mnemonic: "LD A, n8",
                opcode: 0x3E,
                cycles: 8,
            },
            Instruction::LDH_C_A=> InstructionData {
                mnemonic: "LDH [C], A",
                opcode: 0xE2,
                cycles: 8,
            },
            Instruction::INC_C => InstructionData {
                mnemonic: "INC C",
                opcode: 0x0C,
                cycles: 4,
            },
            Instruction::LD_HL_A => InstructionData {
                mnemonic: "LD [HL], A",
                opcode: 0x77,
                cycles: 8,
            },
            Instruction::LDH_a8_A => InstructionData {
                mnemonic: "LDH [a8], A",
                opcode: 0xE0,
                cycles: 12,
            },
            Instruction::LD_B_A => InstructionData {
                mnemonic: "LD B, A",
                opcode: 0x47,
                cycles: 8,
            },
            Instruction::PUSH_HL => InstructionData {
                mnemonic: "PUSH HL",
                opcode: 0xE5,
                cycles: 16,
            },
            Instruction::LD_DE_n16 => InstructionData {
                mnemonic: "LD DE, n16",
                opcode: 0x11,
                cycles: 12,
            },
        }
    }
}

#[cfg(feature = "debug")]
impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let data = self.data();
        write!(f, "{} (0x{:02X?})", data.mnemonic.bright_cyan(), data.opcode)
    }
}

#[repr(u8)]
#[derive(Clone)]
enum Flag {
    Z = 7,
    N = 6,
    H = 5,
    C = 4,
}

struct Registers {
    af: u16,
    bc: u16,
    de: u16,
    hl: u16,
    sp: u16,
    pc: u16,
}
pub struct CPU<T: Drawable> {
    registers: Registers,
    memory: Memory,
    gpu: T,
}

impl<T: Drawable> CPU<T> {
    pub fn new(memory: Memory, gpu: T) -> Self {
        Self {
            registers: Registers {
                af: 0,
                bc: 0,
                de: 0,
                hl: 0,
                sp: 0,
                pc: 0,
            },
            memory,
            gpu,
        }
    }

    #[cfg(not(feature = "debug"))]
    pub fn run(&mut self) {
        let mut cycles = 0;
        let one_sec = Duration::from_secs(1);
        let mut next_cycle = 0;
        loop {
            let timer = Instant::now();
            while cycles < FREQUENCY {
                if next_cycle == cycles {
                    let instruction = self.cycle();
                    next_cycle = cycles + instruction.data().cycles as u32;
                }
                cycles += 1;
            }
            let elapsed = timer.elapsed();
            if elapsed < one_sec {
                thread::sleep(one_sec - elapsed);
            }
            cycles = 0;
        }
    }

    fn cycle(&mut self) -> Instruction {
        let opcode: u8 = self.memory.memory[self.registers.pc as usize];
        let instruction = self.decode(opcode);
        self.gpu.draw();
        instruction
    }

    #[cfg(feature = "debug")]
    pub fn run(&mut self) {
        let debug_mode_msg = "Running in Debug Mode".bright_yellow();
        let help = "help".bold();
        let guide_msg = format!("Type {help} to see the list of commands!");
        println!("");
        println!(" {debug_mode_msg}");
        println!(" {guide_msg}");
        println!("");
        let mut action = String::new();
        loop {
            let debugger_prefix = "(gb-debugger) ".bright_green();
            println!("");
            print!("{debugger_prefix}");
            io::stdout().flush().expect("Failed to flush stdout");
            action.clear();
            io::stdin()
                .read_line(&mut action)
                .expect("Failed to read line");
            println!("");
            match action.trim() {
                "help" => {
                    let mut table = Table::new();
                    table.add_row(row!["Command", "Description"]);
                    table.add_row(row!["run", "Start the emulator and run the loaded ROM."]);
                    table.add_row(row!["quit, q", "Exit the debugger"]);
                    table.add_row(row!["step", "Execute one cycle of the emulator."]);
                    table.add_row(row!["display rom", "Display the current ROM contents."]);
                    table.add_row(row!["show register <REG>", "Show the value of a specific register\n(e.g., af, bc, de, hl, sp, pc or all)."]);
                    table.add_row(row![
                        "show memory <ADDR>",
                        "Display memory content at a given address."
                    ]);
                    table.printstd();
                }
                "run" => {
                    let mut cycles = 0;
                    let one_sec = Duration::from_secs(1);
                    loop {
                        let timer = Instant::now();
                        while cycles < FREQUENCY {
                            println!("{}", self.cycle());
                            cycles += 1;
                        }
                        let elapsed = timer.elapsed();
                        if elapsed < one_sec {
                            thread::sleep(one_sec - elapsed);
                        }
                        cycles = 0;
                    }
                }
                "quit" | "q" => {
                    break;
                }
                "step" => {
                    println!("{}", self.cycle());
                }
                "display rom" => match self.memory.display_rom() {
                    Ok(_) => {}
                    Err(_) => continue,
                },
                cmd if cmd.starts_with("show register ") => {
                    match cmd.trim_start_matches("show register ") {
                        "af" => println!("0x{:02X?}", self.registers.af),
                        "bc" => println!("0x{:02X?}", self.registers.bc),
                        "de" => println!("0x{:02X?}", self.registers.de),
                        "hl" => println!("0x{:02X?}", self.registers.hl),
                        "sp" => println!("0x{:02X?}", self.registers.sp),
                        "pc" => println!("0x{:02X?}", self.registers.pc),
                        "all" => {
                            let mut table = Table::new();
                            table.add_row(row!["AF", format!("0x{:02X?}", self.registers.af)]);
                            table.add_row(row!["BC", format!("0x{:02X?}", self.registers.bc)]);
                            table.add_row(row!["DE", format!("0x{:02X?}", self.registers.de)]);
                            table.add_row(row!["HL", format!("0x{:02X?}", self.registers.hl)]);
                            table.add_row(row!["SP", format!("0x{:02X?}", self.registers.sp)]);
                            table.add_row(row!["PC", format!("0x{:02X?}", self.registers.pc)]);
                            table.printstd();
                        }
                        _ => println!("Unknown register."),
                    }
                }
                cmd if cmd.starts_with("show memory ") => {
                    let parts: Vec<&str> = cmd.split_whitespace().collect();
                    if let Some(addr_str) = parts.get(2) {
                        let address = if addr_str.starts_with("0x") {
                            u16::from_str_radix(&addr_str[2..], 16)
                        } else {
                            addr_str.parse::<u16>()
                        };

                        match address {
                            Ok(address) => {
                                println!(
                                    "Memory at address {}: {}",
                                    addr_str, self.memory.memory[address as usize]
                                );
                            }
                            Err(_) => {
                                println!("Invalid memory address format: {}", addr_str);
                            }
                        }
                    } else {
                        println!("Missing memory address");
                    }
                }

                _ => {
                    println!("{}", action.as_str());
                }
            }
        }
    }
    fn decode(&mut self, opcode: u8) -> Instruction {
        match opcode {
            0x00 => {
                self.registers.pc += 1;
                Instruction::NOP
            }
            0x03 => {
                self.registers.bc = self.registers.bc.wrapping_add(1);
                self.registers.pc += 1;
                Instruction::INC_BC
            }
            0x0B => {
                self.registers.bc = self.registers.bc.wrapping_sub(1);
                self.registers.pc += 1;
                Instruction::DEC_BC
            }
            0x0C => {
                let c = self.get_low_byte(self.registers.bc);
                let result = c.wrapping_add(1);
                self.registers.bc = self.replace_low_byte(self.registers.bc, result); 
                if result == 0 {
                    self.set_flag(Flag::Z);
                }
                self.clear_flag(Flag::N);
                if c & 0x0F == 0x0F {
                    self.set_flag(Flag::H);
                } else {
                    self.clear_flag(Flag::H);
                }
                self.registers.pc += 1;
                Instruction::INC_C
            }
            0x0E => {
                self.registers.bc = self.replace_low_byte(self.registers.bc, self.memory.memory[(self.registers.pc + 1) as usize]);
                self.registers.pc += 2;
                Instruction::LD_C_n8
            }
            0x11 => {
                let low_byte = self.memory.memory[(self.registers.pc + 1) as usize];
                let high_byte = self.memory.memory[(self.registers.pc + 2) as usize];
                self.registers.de = Self::concat_bytes(high_byte, low_byte);
                self.registers.pc += 1;
                Instruction::LD_DE_n16
            }
            0x20 => {
                let mut jump: bool = false;
                if self.get_flag(Flag::Z) == 0 {
                    let e8 = self.memory.memory[(self.registers.pc + 1) as usize] as i8;
                    self.registers.pc = (self.registers.pc as i16 + e8 as i16) as u16;
                    jump = true;
                }
                self.registers.pc += 2;
                Instruction::JR_NZ_e8(jump)
            }
            0x21 => {
                let low = self.memory.memory[(self.registers.pc + 1) as usize];
                let high = self.memory.memory[(self.registers.pc + 2) as usize];
                self.registers.hl = Self::concat_bytes(high, low);
                self.registers.pc += 3;
                Instruction::LD_HL_n16
            }
            0x31 => {
                let low = self.memory.memory[(self.registers.pc + 1) as usize];
                let high = self.memory.memory[(self.registers.pc + 2) as usize];
                self.registers.sp = Self::concat_bytes(high, low);
                self.registers.pc += 3;
                Instruction::LD_SP_n16
            }
            0x32 => {
                self.memory.memory[self.registers.hl as usize] = self.get_high_byte(self.registers.af);
                self.registers.hl -= 1;
                self.registers.pc += 1;
                Instruction::LD_HL_DEC_A
            }
            0x3E => {
                self.registers.af = self.replace_high_byte(self.registers.af, self.memory.memory[(self.registers.pc + 1) as usize]);
                self.registers.pc += 2;
                Instruction::LD_A_n8
            }
            0x47 => {
                let a = self.get_high_byte(self.registers.af);
                self.registers.bc = self.replace_high_byte(self.registers.bc, a);
                self.registers.pc += 1;
                Instruction::LD_B_A
            }
            0x73 => {
                self.memory.memory[self.registers.hl as usize] = self.get_low_byte(self.registers.de);
                self.registers.pc += 1;
                Instruction::LD_HL_E
            }
            0x77 => {
                self.memory.memory[self.registers.hl as usize] = self.get_high_byte(self.registers.af);
                self.registers.pc += 1;
                Instruction::LD_HL_A
            }
            0xCB => {
                let instruction = self.memory.memory[(self.registers.pc + 1) as usize];
                let prefix_opcode = (instruction & 0b1100_0000) >> 6;
                if prefix_opcode == 0 {
                    let cb_opcode = (instruction & 0b0011_1000) >> 3;
                    let operand = self.get_cb_operand(instruction & 0b0000_0111);
                    // TODO: Set appropriate flags
                    match cb_opcode {
                        0x0 => {
                            let carry = (0b1000_0000 & operand) >> 7;
                            let result = (operand << 1) | carry;
                            self.replace_cb_operand(instruction & 0b0000_0111, result);
                            if result == 0 {
                                self.set_flag(Flag::Z);
                            } else {
                                self.clear_flag(Flag::Z);
                            }
                            if carry == 1 {
                                self.set_flag(Flag::C);
                            } else {
                                self.clear_flag(Flag::C);
                            }
                            self.clear_flag(Flag::H);
                            self.clear_flag(Flag::N);
                        }
                        0x1 => {
                            let carry = 0b0000_0001 & operand;
                            let result = (operand >> 1) | carry;
                            if result == 0 {
                                self.set_flag(Flag::Z);
                            } else {
                                self.clear_flag(Flag::Z);
                            }
                            if carry == 1 {
                                self.set_flag(Flag::C);
                            } else {
                                self.clear_flag(Flag::C);
                            }
                            self.clear_flag(Flag::H);
                            self.clear_flag(Flag::N);
                            self.replace_cb_operand(instruction & 0b0000_0111, result);
                        }
                        0x2 => {
                            let carry = (0b1000_0000 & operand) >> 7;
                            let result = operand << 1;
                            self.replace_cb_operand(instruction & 0b0000_0111, result);
                            if result == 0 {
                                self.set_flag(Flag::Z);
                            } else {
                                self.clear_flag(Flag::Z);
                            }
                            if carry == 1 {
                                self.set_flag(Flag::C);
                            } else {
                                self.clear_flag(Flag::C);
                            }
                        }
                        0x3 => {
                            let carry = 0b0000_0001 & operand;
                            let result = operand >> 1;
                            self.replace_cb_operand(instruction & 0b0000_0111, result);
                            if result == 0 {
                                self.set_flag(Flag::Z);
                            } else {
                                self.clear_flag(Flag::Z);
                            }
                            if carry == 1 {
                                self.set_flag(Flag::C);
                            } else {
                                self.clear_flag(Flag::C);
                            }
                        }
                        _ => {
                            panic!("Unknown CB0 opcode: {}", cb_opcode);
                        }
                    }
                } else {
                    let bit_index = (instruction & 0b0011_1000) >> 3;
                    let value = instruction & 0b0000_0111;
                    let operand = self.get_cb_operand(value);
                    match prefix_opcode {
                        0x1 => {
                            let bit = (operand & (1 << bit_index)) >> bit_index;
                            if bit == 1 {
                                self.set_flag(Flag::Z);
                            } else {
                                self.clear_flag(Flag::Z);
                            }
                            self.clear_flag(Flag::N);
                            self.set_flag(Flag::H);
                        }
                        _ => {
                            panic!("Unknown CB1 opcode: {}", prefix_opcode);
                        }
                    }
                }
                self.registers.pc += 2;
                Instruction::PREFIX
            }
            0x66 => {
                let hl = self.registers.hl;
                let h = self.get_high_byte(hl);
                self.registers.hl =
                self.replace_high_byte(hl, self.memory.memory[hl as usize] as u8);
                self.registers.pc += 1;
                Instruction::LD_H_HL
            }
            0xAF => {
                let af = self.registers.af;
                self.registers.af = self.replace_high_byte(af, 0);
                self.set_flag(Flag::Z);
                self.clear_flag(Flag::N);
                self.clear_flag(Flag::H);
                self.clear_flag(Flag::C);
                self.registers.pc += 1;
                Instruction::XOR_A_A
            }
            0xCC => {
                if self.get_flag(Flag::Z) != 0 {
                    let low = self.memory.memory[(self.registers.pc + 1) as usize];
                    let high = self.memory.memory[(self.registers.pc + 2) as usize];
                    let addr = Self::concat_bytes(high, low);
                    self.registers.pc = addr;
                    return Instruction::Call_Z_a16(true);
                } else {
                    self.registers.pc += 3;
                    return Instruction::Call_Z_a16(false);
                }
            }
            0xCE => {
                let af = self.registers.af;
                let a = self.get_high_byte(af);
                let n8 = self.memory.memory[(self.registers.pc + 1) as usize];
                let result = a.wrapping_add(n8).wrapping_add(self.get_flag(Flag::C));
                self.registers.af = self.replace_high_byte(af, result);

                if result == 0 {
                    self.set_flag(Flag::Z);
                }
                self.clear_flag(Flag::N);
                let half_carry = ((a & 0xF) + (n8 & 0xF) + self.get_flag(Flag::C)) > 0x0F;
                if half_carry {
                    self.set_flag(Flag::H);
                }

                let a: u16 = a as u16;
                let n8: u16 = a as u16;
                let result: u16 = (a.wrapping_add(n8).wrapping_add(self.get_flag(Flag::C) as u16)) as u16;
                if result > 0xFF {
                    self.set_flag(Flag::C);
                }
                self.registers.pc += 2;
                Instruction::ADC_A_n8
            }
            0xE0 => {
                let n8 = self.memory.memory[(self.registers.pc + 1) as usize];
                let a8 = 0xFF00 + n8 as u16;
                let a = self.get_high_byte(self.registers.af);
                self.memory.memory[a8 as usize] = a;
                self.registers.pc += 1;
                Instruction::LDH_a8_A
            }
            0xE2 => {
                let c = self.get_low_byte(self.registers.bc);
                self.memory.memory[(self.memory.map.h_ram.start + c as u16) as usize] = self.get_high_byte(self.registers.af);
                self.registers.pc += 1;
                Instruction::LDH_C_A
            }
            0xE5 => {
                self.memory.memory[(self.registers.sp - 1) as usize] = self.get_low_byte(self.registers.hl);
                self.memory.memory[(self.registers.sp - 2) as usize] = self.get_high_byte(self.registers.hl);
                self.registers.sp -= 2;
                self.registers.pc += 1;
                Instruction::PUSH_HL
            }
            _ => todo!(
                "{}",
                format!("Unimplemented opcode: 0x{:02X?} at address 0x{:02X?}", opcode, self.registers.pc).as_str()
            ),
        }
    }
    fn get_high_byte(&self, bytes: u16) -> u8 {
        ((bytes & 0xFF00) >> 8) as u8
    }
    fn get_low_byte(&self, bytes: u16) -> u8 {
        (bytes & 0x00FF) as u8
    }
    fn replace_high_byte(&self, bytes: u16, new_byte: u8) -> u16 {
        (bytes & 0x00FF) | ((new_byte as u16) << 8)
    }
    fn replace_low_byte(&self, bytes: u16, new_byte: u8) -> u16 {
        (bytes & 0xFF00) | (new_byte as u16)
    }
    fn get_flag(&self, flag: Flag) -> u8 {
        ((self.registers.af & (1 << flag.clone() as u8)) >> flag.clone() as u8) as u8
    }
    fn set_flag(&mut self, flag: Flag) {
        let mut flags = (self.registers.af & 0x00FF) as u8;
        flags = flags | (1 << flag.clone() as u8);
        self.registers.af |= flags as u16;
    }
    fn clear_flag(&mut self, flag: Flag) {
        let mut flags = (self.registers.af & 0x00FF) as u8;
        let mask = 1 << flag.clone() as u8;
        let a = (self.get_high_byte(self.registers.af) as u16) << 8;
        self.registers.af = a | ((flags | mask) ^ mask) as u16;
    }
    fn concat_bytes(high: u8, low: u8) -> u16 {
        ((high as u16) << 8) | low as u16
    }
    fn get_leftmost_five_bits(instruction: u8) -> u8 {
        (instruction & 0b1111_1000) >> 3
    }
    fn get_cb_operand(&self, value: u8) -> u8 {
        match value {
            0x0 => {
                self.get_high_byte(self.registers.bc)
            }
            0x1 => {
                self.get_low_byte(self.registers.bc)
            }
            0x2 => {
                self.get_high_byte(self.registers.de)
            }
            0x3 => {
                self.get_low_byte(self.registers.de)
            }
            0x4 => {
                self.get_high_byte(self.registers.hl)
            }
            0x5 => {
                self.get_low_byte(self.registers.hl)
            }
            0x6 => {
                self.memory.memory[self.registers.hl as usize]
            }
            0x7 => {
                self.get_high_byte(self.registers.af)
            }
            _ => {0}
        }
    }
    fn replace_cb_operand(&mut self, operand: u8, value: u8) {
        match operand {
            0x0 => {
                self.registers.bc = self.replace_high_byte(self.registers.bc, value);
            }
            0x1 => {
                self.registers.bc = self.replace_low_byte(self.registers.bc, value);
            }
            0x2 => {
                self.registers.de = self.replace_high_byte(self.registers.de, value);
            }
            0x3 => {
                self.registers.de = self.replace_low_byte(self.registers.de, value);
            }
            0x4 => {
                self.registers.hl = self.replace_high_byte(self.registers.hl, value);
            }
            0x5 => {
                self.registers.hl = self.replace_low_byte(self.registers.hl, value);
            }
            0x6 => {
                self.memory.memory[self.registers.hl as usize] = value;
            }
            0x7 => {
                self.registers.af = self.replace_high_byte(self.registers.af, value);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gpu::GPU;

    struct FakeGPU {}
    impl Drawable for FakeGPU {
        fn draw(&mut self) {}
    }

    fn cpu() -> CPU<FakeGPU> {
        let mem = Memory::new();
        let gpu = FakeGPU {};
        CPU::new(mem, gpu)
    }

    fn assert_flags(cpu: &CPU<FakeGPU>, z: bool, n: bool, h: bool, c: bool) {
        assert_eq!(cpu.get_flag(Flag::Z), z as u8);
        assert_eq!(cpu.get_flag(Flag::N), n as u8);
        assert_eq!(cpu.get_flag(Flag::H), h as u8);
        assert_eq!(cpu.get_flag(Flag::C), c as u8);
    }

    #[test]
    fn should_return_cb_opcode() {
        assert_eq!(CPU::<FakeGPU>::get_leftmost_five_bits(0b00000_0000), 0b0000_0000);
        assert_eq!(CPU::<FakeGPU>::get_leftmost_five_bits(0b0000_1000), 0b0000_0001);
        assert_eq!(CPU::<FakeGPU>::get_leftmost_five_bits(0b0001_0000), 0b0000_0010);
        assert_eq!(CPU::<FakeGPU>::get_leftmost_five_bits(0b0001_1000), 0b0000_0011);
        assert_eq!(CPU::<FakeGPU>::get_leftmost_five_bits(0b0010_0000), 0b0000_0100);
        assert_eq!(CPU::<FakeGPU>::get_leftmost_five_bits(0b0010_1000), 0b0000_0101);
        assert_eq!(CPU::<FakeGPU>::get_leftmost_five_bits(0b0011_0000), 0b0000_0110);
        assert_eq!(CPU::<FakeGPU>::get_leftmost_five_bits(0b0011_1000), 0b0000_0111);
    }

    #[test]
    fn should_return_high_byte() {
        let cpu = cpu();
        assert_eq!(cpu.get_high_byte(0xFF00), 0xFF);
        assert_eq!(cpu.get_high_byte(0x00FF), 0x00);
        assert_eq!(cpu.get_high_byte(0xCE03), 0xCE);
        assert_eq!(cpu.get_high_byte(0xAF30), 0xAF);
    }

    #[test]
    fn should_return_low_byte() {
        let cpu = cpu();
        assert_eq!(cpu.get_low_byte(0xFF00), 0x00);
        assert_eq!(cpu.get_low_byte(0x00FF), 0xFF);
        assert_eq!(cpu.get_low_byte(0x1ABC), 0xBC);
        assert_eq!(cpu.get_low_byte(0xCA12), 0x12);
    }

    #[test]
    fn should_replace_high_byte() {
        let cpu = cpu();
        assert_eq!(cpu.replace_high_byte(0xFF00, 0xAC), 0xAC00);
        assert_eq!(cpu.replace_high_byte(0x0000, 0xFF), 0xFF00);
        assert_eq!(cpu.replace_high_byte(0xAB34, 0xDA), 0xDA34);
    }

    #[test]
    fn should_replace_low_byte() {
        let cpu = cpu();
        assert_eq!(cpu.replace_low_byte(0xABCD, 0xEF), 0xABEF);
    }

    #[test]
    fn should_correctly_return_flag_values_from_af_register() {
        let mut cpu = cpu();
        cpu.registers.af = 0b00000000_10010000;
        assert_eq!(cpu.get_flag(Flag::Z), 1);
        assert_eq!(cpu.get_flag(Flag::N), 0);
        assert_eq!(cpu.get_flag(Flag::H), 0);
        assert_eq!(cpu.get_flag(Flag::C), 1);
    }

    #[test]
    fn should_set_corresponding_flags_in_af_register() {
        let mut cpu = cpu();

        cpu.registers.af = 0x00;
        cpu.set_flag(Flag::Z);
        assert_eq!(cpu.registers.af, 0b00000000_10000000);

        cpu.registers.af = 0x00;
        cpu.set_flag(Flag::N);
        assert_eq!(cpu.registers.af, 0b00000000_01000000);

        cpu.registers.af = 0x00;
        cpu.set_flag(Flag::H);
        assert_eq!(cpu.registers.af, 0b00000000_00100000);

        cpu.registers.af = 0x00;
        cpu.set_flag(Flag::C);
        assert_eq!(cpu.registers.af, 0b00000000_00010000);
    }

    #[test]
    fn should_clear_corresponding_flags_in_af_register() {
        let mut cpu = cpu();

        cpu.registers.af = 0x00FF;
        cpu.clear_flag(Flag::Z);
        assert_eq!(cpu.registers.af, 0b00000000_01111111);

        cpu.registers.af = 0x00FF;
        cpu.clear_flag(Flag::N);
        assert_eq!(cpu.registers.af, 0b00000000_10111111);

        cpu.registers.af = 0x00FF;
        cpu.clear_flag(Flag::H);
        assert_eq!(cpu.registers.af, 0b00000000_11011111);

        cpu.registers.af = 0x00FF;
        cpu.clear_flag(Flag::C);
        assert_eq!(cpu.registers.af, 0b00000000_11101111);
    }

    #[test]
    fn should_concatenate_bytes() {
        assert_eq!(CPU::<FakeGPU>::concat_bytes(0x10, 0xAC), 0x10AC);
    }

    #[test]
    // 0xCE
    fn adc_a_n8() {
        let mut cpu = cpu();

        cpu.registers.pc = 0;
        cpu.memory.memory[1] = 5;
        cpu.registers.af = 0x0100;
        assert_eq!(Instruction::ADC_A_n8, cpu.decode(0xCE));
        assert_eq!(cpu.registers.pc, 2);
        assert_eq!(cpu.registers.af, 0x0600);

        cpu.registers.pc = 0;
        cpu.memory.memory[1] = 1;
        cpu.registers.af = 0xFF00;
        assert_eq!(Instruction::ADC_A_n8, cpu.decode(0xCE));
        assert_flags(&cpu, true, false, true, true);
    }

    #[test]
    // 0x66
    fn ld_h_hl() {
        let mut cpu = cpu();
        cpu.registers.pc = 0;
        cpu.registers.hl = 0xFF02;
        cpu.memory.memory[0xFF02] = 0xA;
        assert_eq!(Instruction::LD_H_HL, cpu.decode(0x66));
        assert_eq!(cpu.registers.pc, 1);
        assert_eq!(cpu.registers.hl, 0x0A02);
    }

    #[test]
    // 0xCC
    fn call_z_a16() {
        let mut cpu = cpu();
        cpu.registers.af = 0xFF;
        cpu.registers.pc = 0;
        cpu.memory.memory[1] = 0xCD;
        cpu.memory.memory[2] = 0xAB;
        assert_eq!(Instruction::Call_Z_a16(true), cpu.decode(0xCC));
        assert_eq!(cpu.registers.pc, 0xABCD);


        cpu.registers.af = 0x00;
        cpu.registers.pc = 0;
        cpu.memory.memory[1] = 0xCD;
        cpu.memory.memory[2] = 0xAB;
        assert_eq!(Instruction::Call_Z_a16(false), cpu.decode(0xCC));
        assert_eq!(cpu.registers.pc, 3);
    }

    #[test]
    // 0x0B
    fn dec_bc() {
        let mut cpu = cpu();
        cpu.registers.pc = 0;
        cpu.registers.bc = 0x02;
        assert_eq!(Instruction::DEC_BC, cpu.decode(0x0B));
        assert_eq!(cpu.registers.bc, 0x01);
        assert_eq!(cpu.registers.pc, 1);
    }

    #[test]
    // 0x03
    fn inc_bc() {
        let mut cpu = cpu();
        cpu.registers.pc = 0;
        cpu.registers.bc = 0x01;
        assert_eq!(Instruction::INC_BC, cpu.decode(0x03));
        assert_eq!(cpu.registers.bc, 0x02);
        assert_eq!(cpu.registers.pc, 1);
    }

    #[test]
    // 0x73
    fn ld_hl_e() {
        let mut cpu = cpu();
        cpu.registers.pc = 0;
        cpu.registers.hl = 0x00;
        cpu.memory.memory[cpu.registers.hl as usize] = 0x01;
        cpu.registers.de = 0xAB;
        assert_eq!(Instruction::LD_HL_E, cpu.decode(0x73));
        assert_eq!(cpu.memory.memory[cpu.registers.hl as usize], 0xAB);
        assert_eq!(cpu.registers.pc, 1);
    }

    #[test]
    // 0x0
    fn nop() {
        let mut cpu = cpu();
        cpu.registers.pc = 0;
        assert_eq!(Instruction::NOP, cpu.decode(0x00));
        assert_eq!(cpu.registers.pc, 1);
    }

    #[test]
    // 0x0E
    fn ld_c_n8() {
        let mut cpu = cpu();
        cpu.registers.bc = 0xABCD;
        cpu.registers.pc = 0;
        cpu.memory.memory[(cpu.registers.pc + 1) as usize] = 0xEF;
        assert_eq!(Instruction::LD_C_n8, cpu.decode(0x0E));
        assert_eq!(cpu.registers.bc, 0xABEF);
    }

    #[test]
    // 0x20
    fn jr_nz_e8() {
        let mut cpu = cpu();
        cpu.registers.pc = 2;
        cpu.registers.af = 0b00000000_10000000;
        cpu.memory.memory[(cpu.registers.pc + 1) as usize] = 0xFF;
        assert_eq!(Instruction::JR_NZ_e8(false), cpu.decode(0x20));
        assert_eq!(cpu.registers.pc, 4);


        cpu.registers.pc = 2;
        cpu.registers.af = 0;
        cpu.memory.memory[(cpu.registers.pc + 1) as usize] = 0xFF;
        assert_eq!(Instruction::JR_NZ_e8(true), cpu.decode(0x20));
        assert_eq!(cpu.registers.pc, 3);
    } 

    #[test]
    // 0x21
    fn ld_hl_n16() {
        let mut cpu = cpu();
        cpu.registers.pc = 0;
        cpu.registers.hl = 0;
        cpu.memory.memory[(cpu.registers.pc + 1) as usize] = 0xCD;
        cpu.memory.memory[(cpu.registers.pc + 2) as usize] = 0xAB;
        assert_eq!(Instruction::LD_HL_n16, cpu.decode(0x21));
        assert_eq!(cpu.registers.hl, 0xABCD);
    }

    #[test]
    // 0x31
    fn ld_sp_n16() {
        let mut cpu = cpu();
        cpu.registers.pc = 0;
        cpu.registers.sp = 0;
        cpu.memory.memory[(cpu.registers.pc + 1) as usize] = 0xCD;
        cpu.memory.memory[(cpu.registers.pc + 2) as usize] = 0xAB;
        assert_eq!(Instruction::LD_SP_n16, cpu.decode(0x31));
        assert_eq!(cpu.registers.sp, 0xABCD);
    }

    #[test]
    // 0x32
    fn ld_hl_dec_a() {
        let mut cpu = cpu();
        cpu.registers.hl = 2;
        cpu.registers.af = 0xABCD;
        cpu.memory.memory[cpu.registers.hl as usize] = 0;
        assert_eq!(Instruction::LD_HL_DEC_A, cpu.decode(0x32));
        assert_eq!(cpu.memory.memory[(cpu.registers.hl + 1) as usize], 0xAB);
        assert_eq!(cpu.registers.hl, 1);
    }

    #[test]
    // 0xAF
    fn xor_a_a() {
        let mut cpu = cpu();
        cpu.registers.af = 0xAB00;
        assert_eq!(Instruction::XOR_A_A, cpu.decode(0xAF));
        assert_eq!(0x0080, cpu.registers.af);
        assert_flags(&cpu, true, false, false, false);
    }

    #[test]
    // 0x3E
    fn ld_a_n8() {
        let mut cpu = cpu();
        cpu.registers.af = 0xABCD;
        cpu.registers.pc = 0;
        cpu.memory.memory[(cpu.registers.pc + 1) as usize] = 0xEF;
        assert_eq!(Instruction::LD_A_n8, cpu.decode(0x3E));
        assert_eq!(cpu.registers.af, 0xEFCD);
    }
    
    #[test]
    // 0xE2
    fn ldh_c_a() {
        let mut cpu = cpu();
        cpu.registers.bc = 0xAB01;
        cpu.registers.af = 0xFF00;
        assert_eq!(Instruction::LDH_C_A, cpu.decode(0xE2));
        assert_eq!(cpu.memory.memory[(cpu.memory.map.h_ram.start + 0x01) as usize], 0xFF);
    }

    #[test]
    // 0x77
    fn ld_hl_a() {
        let mut cpu = cpu();
        cpu.registers.af = 0xFF00;
        cpu.registers.hl = 0x1122;
        assert_eq!(Instruction::LD_HL_A, cpu.decode(0x77));
        assert_eq!(cpu.memory.memory[cpu.registers.hl as usize], 0xFF);
    }

    #[test]
    fn ldh_a8_a() {
        let mut cpu = cpu();
        cpu.registers.af = 0xFF00;
        cpu.memory.memory[(cpu.registers.pc + 1) as usize] = 0xAB;
        assert_eq!(Instruction::LDH_a8_A, cpu.decode(0xE0));
        assert_eq!(cpu.memory.memory[0xFFAB], 0xFF);
    }

    #[test]
    fn ld_b_a() {
        let mut cpu = cpu();
        cpu.registers.af = 0xAA00;
        cpu.registers.bc = 0xBB00;
        assert_eq!(Instruction::LD_B_A, cpu.decode(0x47));
        assert_eq!(cpu.registers.bc, 0xAA00);
    }

    #[test]
    fn cb_rlc() {
        let mut cpu = cpu();
        cpu.registers.bc = 0b1000_0000_0000_0000;
        cpu.memory.memory[0] = 0xCB;
        cpu.memory.memory[1] = 0x0;
        cpu.registers.pc = 0;
        assert_eq!(Instruction::PREFIX, cpu.decode(0xCB));
        assert_eq!(cpu.registers.bc, 0b0000_0001_0000_0000);
        assert_flags(&cpu, false, false, false, true);
    }

    #[test]
    fn cb_rrc() {
        let mut cpu = cpu();
        cpu.registers.bc = 0b1000_0000_0000_0000;
        cpu.memory.memory[0] = 0xCB;
        cpu.memory.memory[1] = 0b0000_1000;
        cpu.registers.pc = 0;
        assert_eq!(Instruction::PREFIX, cpu.decode(0xCB));
        assert_eq!(cpu.registers.bc, 0b0100_0000_0000_0000);
        assert_flags(&cpu, false, false, false, false);
    }

    #[test]
    fn cb_rl() {
        let mut cpu = cpu();
        cpu.registers.bc = 0b1000_0000_0000_0000;
        cpu.memory.memory[0] = 0xCB;
        cpu.memory.memory[1] = 0b0001_0000;
        cpu.registers.pc = 0;
        assert_eq!(Instruction::PREFIX, cpu.decode(0xCB));
        assert_eq!(cpu.registers.bc, 0);
        assert_flags(&cpu, true, false, false, true);
    }
    #[test]
    fn cb_rr() {
        let mut cpu = cpu();
        cpu.registers.bc = 0b0000_0001_0000_0000;
        cpu.memory.memory[0] = 0xCB;
        cpu.memory.memory[1] = 0b0001_1000;
        cpu.registers.pc = 0;
        assert_eq!(Instruction::PREFIX, cpu.decode(0xCB));
        assert_eq!(cpu.registers.bc, 0);
        assert_flags(&cpu, true, false, false, true);
    }
}
