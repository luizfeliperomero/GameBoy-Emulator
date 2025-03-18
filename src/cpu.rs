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
                pc: 0x0104,
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
            0x66 => {
                let hl = self.registers.hl;
                let h = self.get_high_byte(hl);
                self.registers.hl =
                self.replace_high_byte(hl, self.memory.memory[hl as usize] as u8);
                self.registers.pc += 1;
                Instruction::LD_H_HL
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
            0x0B => {
                self.registers.bc = self.registers.bc.wrapping_sub(1);
                self.registers.pc += 1;
                Instruction::DEC_BC
            }
            0x03 => {
                self.registers.bc = self.registers.bc.wrapping_add(1);
                self.registers.pc += 1;
                Instruction::INC_BC
            }
            0x73 => {
                self.memory.memory[self.registers.hl as usize] = self.get_low_byte(self.registers.de);
                self.registers.pc += 1;
                Instruction::LD_HL_E
            }
            0x00 => {
                self.registers.pc += 1;
                Instruction::NOP
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
    fn adc_a_n8() {
        let mut cpu = cpu();

        cpu.registers.pc = 0;
        cpu.memory.memory[1] = 5;
        cpu.registers.af = 0x0100;
        let instruction = cpu.decode(0xCE);
        assert_eq!(instruction, Instruction::ADC_A_n8);
        assert_eq!(cpu.registers.pc, 2);
        assert_eq!(cpu.registers.af, 0x0600);

        cpu.registers.pc = 0;
        cpu.memory.memory[1] = 1;
        cpu.registers.af = 0xFF00;
        cpu.decode(0xCE);
        assert_eq!(cpu.get_flag(Flag::Z), 1);
        assert_eq!(cpu.get_flag(Flag::N), 0);
        assert_eq!(cpu.get_flag(Flag::C), 1);
        assert_eq!(cpu.get_flag(Flag::H), 1);
    }

    #[test]
    fn ld_h_hl() {
        let mut cpu = cpu();
        cpu.registers.pc = 0;
        cpu.registers.hl = 0xFF02;
        cpu.memory.memory[0xFF02] = 0xA;
        let instruction = cpu.decode(0x66);
        assert_eq!(instruction, Instruction::LD_H_HL);
        assert_eq!(cpu.registers.pc, 1);
        assert_eq!(cpu.registers.hl, 0x0A02);
    }

    #[test]
    fn call_z_a16() {
        let mut cpu = cpu();
        cpu.registers.af = 0xFF;
        cpu.registers.pc = 0;
        cpu.memory.memory[1] = 0xCD;
        cpu.memory.memory[2] = 0xAB;
        cpu.decode(0xCC);
        assert_eq!(cpu.registers.pc, 0xABCD);


        cpu.registers.af = 0x00;
        cpu.registers.pc = 0;
        cpu.memory.memory[1] = 0xCD;
        cpu.memory.memory[2] = 0xAB;
        cpu.decode(0xCC);
        assert_eq!(cpu.registers.pc, 3);
    }

    #[test]
    fn dec_bc() {
        let mut cpu = cpu();
        cpu.registers.pc = 0;
        cpu.registers.bc = 0x02;
        cpu.decode(0x0B);
        assert_eq!(cpu.registers.bc, 0x01);
        assert_eq!(cpu.registers.pc, 1);
    }

    #[test]
    fn inc_bc() {
        let mut cpu = cpu();
        cpu.registers.pc = 0;
        cpu.registers.bc = 0x01;
        cpu.decode(0x03);
        assert_eq!(cpu.registers.bc, 0x02);
        assert_eq!(cpu.registers.pc, 1);
    }

    #[test]
    fn ld_hl_e() {
        let mut cpu = cpu();
        cpu.registers.pc = 0;
        cpu.registers.hl = 0x00;
        cpu.memory.memory[cpu.registers.hl as usize] = 0x01;
        cpu.registers.de = 0xAB;
        cpu.decode(0x73);
        assert_eq!(cpu.memory.memory[cpu.registers.hl as usize], 0xAB);
        assert_eq!(cpu.registers.pc, 1);
    }
}
