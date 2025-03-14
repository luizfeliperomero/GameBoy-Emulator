#[cfg(feature = "debug")]
#[macro_use] extern crate prettytable;
extern crate sdl2;

use crate::cpu::CPU;
use crate::memory::Memory;
use crate::gpu::GPU;
pub mod cpu;
pub mod memory;
pub mod gpu;

fn main() {
    let mut mem = Memory::new();
    match mem.load_rom("roms/super-mario-land.gb") {
        Ok(_) => {
            let mut gpu = GPU::new();
            let mut cpu = CPU::new(mem, gpu);
            cpu.run();
        },
        Err(error) => panic!("Problem reading file: {error:?}"),
    };
}
