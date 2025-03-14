use std::time::{Instant, Duration};
use std::thread;
use crate::memory::Memory;

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
pub struct CPU {
    registers: Registers,
    memory: Memory
}

impl CPU {
    pub fn new(memory: Memory) -> Self {
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
        }
    }
    pub fn run(&mut self) {
        let mut cycles = 0;
        let one_sec = Duration::from_secs(1);
        loop {
            let timer = Instant::now();
            while cycles < FREQUENCY {
                self.decode();
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
        let instruction_byte = self.memory.memory[self.registers.pc as usize];
        println!("{:02X?}", instruction_byte);
    }
}
