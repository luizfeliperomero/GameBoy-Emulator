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
}

impl fmt::Display for Instruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let output = match self {
            Instruction::ADC_A_n8 => {
                let mnemonic = "ADC A, n8".bright_cyan();
                let opcode = "0xCE";
                format!("{mnemonic} ({opcode})")
            }
            Instruction::LD_H_HL => {
                let mnemonic = "LD H, [HL]".bright_cyan();
                let opcode = "0x66";
                format!("{mnemonic} ({opcode})")
            }
        };
        write!(f, "{}", output)
    }
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
        loop {
            let timer = Instant::now();
            while cycles < FREQUENCY {
                self.cycle();
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
                let a = self.get_leftmost_byte(af);
                let n8 = self.memory.memory[(self.registers.pc + 1) as usize];
                let result = a.wrapping_add(n8).wrapping_add(self.get_carry_flag());
                self.registers.af = self.replace_leftmost_byte(af, result);
                if result == 0 {
                    self.set_zero_flag();
                }
                self.clear_n_flag();
                let half_carry = (a & 0xF) + (n8 & 0xF) + self.get_carry_flag() > 0x0F;
                if half_carry {
                    self.set_h_flag();
                }
                let result_u16: u16 = (a + n8 + self.get_carry_flag()) as u16;
                if result_u16 > 0xFF {
                    self.set_carry_flag();
                }
                self.registers.pc += 2;
                Instruction::ADC_A_n8
            }
            0x66 => {
                let hl = self.registers.hl;
                let h = self.get_leftmost_byte(hl);
                self.registers.hl =
                    self.replace_leftmost_byte(hl, self.memory.memory[hl as usize] as u8);
                self.registers.pc += 1;
                Instruction::LD_H_HL
            }
            _ => todo!(
                "{}",
                format!("Unimplemented opcode: {:02X?}", opcode).as_str()
            ),
        }
    }
    fn get_leftmost_byte(&self, bytes: u16) -> u8 {
        ((bytes & 0xFF00) >> 8) as u8
    }
    fn get_rightmost_byte(&self, bytes: u16) -> u8 {
        (bytes & 0x00FF) as u8
    }
    fn replace_leftmost_byte(&self, bytes: u16, new_byte: u8) -> u16 {
        (bytes & 0x00FF) | ((new_byte as u16) << 8)
    }
    fn get_carry_flag(&self) -> u8 {
        let flags = (self.registers.af & 0x00FF) as u8;
        (flags & (1 << 4)) >> 4
    }
    fn set_zero_flag(&mut self) {
        let mut flags = (self.registers.af & 0x00FF) as u8;
        flags = flags | (1 << 7);
        self.registers.af |= flags as u16;
    }
    fn clear_n_flag(&mut self) {
        let mut flags = (self.registers.af & 0x00FF) as u8;
        let mask = 1 << 6;
        let a = (self.get_leftmost_byte(self.registers.af) as u16) << 8;
        self.registers.af = a | ((flags | mask) ^ mask) as u16;
    }
    fn set_h_flag(&mut self) {
        let mut flags = (self.registers.af & 0x00FF) as u8;
        flags = flags | (1 << 5);
        self.registers.af |= flags as u16;
    }
    fn set_carry_flag(&mut self) {
        let mut flags = (self.registers.af & 0x00FF) as u8;
        flags = flags | (1 << 4);
        self.registers.af |= flags as u16;
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
    fn should_return_leftmost_byte() {
        let cpu = cpu();
        assert_eq!(cpu.get_leftmost_byte(0xFF00), 0xFF);
        assert_eq!(cpu.get_leftmost_byte(0x00FF), 0x00);
        assert_eq!(cpu.get_leftmost_byte(0xCE03), 0xCE);
        assert_eq!(cpu.get_leftmost_byte(0xAF30), 0xAF);
    }

    #[test]
    fn should_return_rightmost_byte() {
        let cpu = cpu();
        assert_eq!(cpu.get_rightmost_byte(0xFF00), 0x00);
        assert_eq!(cpu.get_rightmost_byte(0x00FF), 0xFF);
        assert_eq!(cpu.get_rightmost_byte(0x1ABC), 0xBC);
        assert_eq!(cpu.get_rightmost_byte(0xCA12), 0x12);
    }

    #[test]
    fn should_replace_leftmost_byte() {
        let cpu = cpu();
        assert_eq!(cpu.replace_leftmost_byte(0xFF00, 0xAC), 0xAC00);
        assert_eq!(cpu.replace_leftmost_byte(0x0000, 0xFF), 0xFF00);
        assert_eq!(cpu.replace_leftmost_byte(0xAB34, 0xDA), 0xDA34);
    }

    #[test]
    fn should_return_carry_flag() {
        let mut cpu = cpu();
        cpu.registers.af = 0b0001_0000;
        assert_eq!(cpu.get_carry_flag(), 1);
        cpu.registers.af = 0b0000_0000;
        assert_eq!(cpu.get_carry_flag(), 0);
    }

    #[test]
    fn should_set_zero_flag() {
        let mut cpu = cpu();
        cpu.registers.af = 0b11111111_00000000;
        cpu.set_zero_flag();
        assert_eq!(cpu.registers.af, 0b11111111_10000000);
        cpu.registers.af = 0b01010010_10101010;
        cpu.set_zero_flag();
        assert_eq!(cpu.registers.af, 0b01010010_10101010);
    }

    #[test]
    fn should_clear_n_flag() {
        let mut cpu = cpu();
        cpu.registers.af = 0b11111111_11111111;
        cpu.clear_n_flag();
        assert_eq!(cpu.registers.af, 0b11111111_10111111);
        cpu.registers.af = 0b11111111_10111111;
        cpu.clear_n_flag();
        assert_eq!(cpu.registers.af, 0b11111111_10111111);
    }

    #[test]
    fn should_set_h_flag() {
        let mut cpu = cpu();
        cpu.registers.af = 0b11111111_11011111;
        cpu.set_h_flag();
        assert_eq!(cpu.registers.af, 0b11111111_11111111);
        cpu.registers.af = 0b11111111_11111111;
        cpu.set_h_flag();
        assert_eq!(cpu.registers.af, 0b11111111_11111111);
    }

    #[test]
    fn should_set_carry_flag() {
        let mut cpu = cpu();
        cpu.registers.af = 0b00000000_00000000;
        cpu.set_carry_flag();
        assert_eq!(cpu.registers.af, 0b00000000_00010000);
        cpu.registers.af = 0b00000000_00010000;
        cpu.set_carry_flag();
        assert_eq!(cpu.registers.af, 0b00000000_00010000);
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
}
