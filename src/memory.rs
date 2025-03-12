const MEMORY_SIZE: u16 = 2.pow(16);
struct Range {
    start: u8,
    end: u8,
}
impl Range {
    fn new(start: u8, end: u8) -> Self {
        Self {
            start,
            end
        }
    }
}
enum MemoryMap {
    ROM(Range),
    VRAM(Range),
    ExternalRAM(Range),
    WorkRAM(Range),
    OAM(Range),
    IO(Range),
    HRAM(Range),
}
pub struct Memory {
    memory: [u8; MEMORY_SIZE],
    map: [MemoryMap, 7]
}
impl Memory {
    pub fn new() -> Self {
        Self {
            memory: [0u8; MEMORY_SIZE],
            maps: [
                ROM(Range::new(0x0000, 0x7FFF)),
                VRAM(Range::new(0x8000, 0x9FFF)),
                ExternalRAM(Range::new(0xA000, 0xBFFF)),
                WorkRAM(Range::new(0xC000, 0xDFFF)),
                OAM(Range::new(0xFE00, 0xFE9F)),
                IO(Range::new(0xFF00, 0xFF7F)),
                HRAM(Range::new(0xFF80, 0xFFFE)),
            ]
        }
    }
}
