enum Register {
    AF(u16),
    BC(u16),
    DE(u16),
    HL(u16),
    SP(u16),
    PC(u16),
    FLAGS(u8)
}
pub struct CPU {
    registers: [Register; 7]
}

impl CPU {
    pub fn new() -> Self {
        Self {
             registers: [
                Register::AF(0),
                Register::BC(0),
                Register::DE(0),
                Register::HL(0),
                Register::SP(0),
                Register::PC(0),
                Register::FLAGS(0)
            ]
        }
    }
}
