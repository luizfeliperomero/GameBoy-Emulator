use crate::gpu::Drawable;
use crate::memory::Memory;
use std::thread;
use std::time::{Duration, Instant};

const FREQUENCY: u32 = 4_194_304;

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
    pub fn run(&mut self) {
        let mut cycles = 0;
        let one_sec = Duration::from_secs(1);
        loop {
            let timer = Instant::now();
            while cycles < FREQUENCY {
                self.decode();
                self.gpu.draw();
                cycles += 1;
            }
            let elapsed = timer.elapsed();
            if elapsed < one_sec {
                thread::sleep(one_sec - elapsed);
            }
            cycles = 0;
        }
    }
    fn decode(&mut self) {
        let opcode: u8 = self.memory.memory[self.registers.pc as usize];
        match opcode {
            0xCE => {
                let af = self.registers.af;
                let a = self.get_leftmost_byte(af);
                let n8 = self.memory.memory[(self.registers.pc + 1) as usize];
                let ac = a + n8 + self.get_carry_flag();
                self.registers.pc += 2;
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
}
