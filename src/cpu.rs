use std::time::{Instant, Duration};
use std::thread;
use crate::memory::Memory;
use crate::gpu::Drawable;

const FREQUENCY: u32 = 4_194_304;

struct Registers {
    af: u16,
    bc: u16,
    de: u16,
    hl: u16,
    sp: u16,
    pc: u16,
    flags: u8
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
                flags: 0
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
            _ => todo!("{}", format!("Unimplemented opcode: {:02X?}", opcode).as_str())
        }
    }
    fn get_leftmost_byte(&self, bytes: u16) -> u8 {
        ((bytes & 0xFF00) >> 8) as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gpu::GPU;

    struct FakeGPU{}
    impl Drawable for FakeGPU {
        fn draw(&mut self){}
    }

    fn cpu() -> CPU<FakeGPU> {
        let mem = Memory::new();
        let gpu = FakeGPU{};
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

}
