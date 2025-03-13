use std::fs;
use std::error::Error;

const MEMORY_SIZE: usize = 2_usize.pow(16);
struct Range {
    start: u16,
    end: u16,
}
impl Range {
    fn new(start: u16, end: u16) -> Self {
        Self {
            start,
            end
        }
    }
}
struct MemoryMap {
    rom: Range,
    v_ram: Range,
    external_ram: Range,
    work_ram: Range,
    oam: Range,
    io: Range,
    h_ram: Range,
}
pub struct Memory {
    memory: [u8; MEMORY_SIZE],
    map: MemoryMap,
    rom_size: usize
}
impl Memory {
    pub fn new() -> Self {
        Self {
            memory: [0u8; MEMORY_SIZE],
            map: MemoryMap {
                rom: Range::new(0x0000, 0x7FFF),
                v_ram: Range::new(0x8000, 0x9FFF),
                external_ram: Range::new(0xA000, 0xBFFF),
                work_ram: Range::new(0xC000, 0xDFFF),
                oam: Range::new(0xFE00, 0xFE9F),
                io: Range::new(0xFF00, 0xFF7F),
                h_ram: Range::new(0xFF80, 0xFFFE),
            },
            rom_size: 0
        }
    }
}
