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
    registers: Registers
}

impl CPU {
    pub fn new() -> Self {
        Self {
            registers: Registers {
                af: 0,
                bc: 0,
                de: 0,
                hl: 0,
                sp: 0,
                pc: 0,
                flags: 0
            }
        }
    }
}
