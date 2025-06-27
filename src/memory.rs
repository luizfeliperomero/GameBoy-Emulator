#[cfg(feature = "debug")]
use prettytable::{Cell, Row, Table, format};
#[cfg(feature = "debug")]
use std::fmt::Write as _;
#[cfg(feature = "debug")]
use std::io::Write;
#[cfg(feature = "debug")]
use std::process::{Command, Stdio};

use std::error::Error;
use std::fs;

const MEMORY_SIZE: usize = 2_usize.pow(16);
pub struct Range {
    pub start: u16,
    pub end: u16,
}
impl Range {
    fn new(start: u16, end: u16) -> Self {
        Self { start, end }
    }
}
pub struct MemoryMap {
    rom: Range,
    v_ram: Range,
    external_ram: Range,
    work_ram: Range,
    oam: Range,
    io: Range,
    pub h_ram: Range,
}
pub struct Memory {
    pub memory: [u8; MEMORY_SIZE],
    pub map: MemoryMap,
    rom_size: usize,
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
            rom_size: 0,
        }
    }
    pub fn load_rom(&mut self, path: &str) -> Result<(), Box<dyn Error>> {
        let file = fs::read(path)?;
        file.iter()
            .enumerate()
            .for_each(|(i, byte)| self.memory[i] = *byte);
        self.rom_size = file.len();
        Ok(())
    }

    #[cfg(feature = "debug")]
    pub fn display_rom(&self) -> Result<(), std::io::Error> {
        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_NO_LINESEP_WITH_TITLE);
        let cols_per_row = 16;

        let mut headers = vec![Cell::new("Addr (0x)")];
        headers.extend((0..cols_per_row).map(|i| Cell::new(&format!("{:X}", i))));
        table.set_titles(Row::new(headers));

        for (i, chunk) in self.memory[..self.rom_size]
            .chunks(cols_per_row)
            .enumerate()
        {
            let mut row = vec![Cell::new(&format!("{:04X}", i * cols_per_row))];

            for &byte in chunk {
                row.push(Cell::new(&format!("{:02X}", byte)));
            }

            while row.len() < cols_per_row + 1 {
                row.push(Cell::new("  "));
            }

            table.add_row(Row::new(row));
        }

        // Capture output as a string
        let mut output = Vec::new();
        table.print(&mut output).unwrap(); // `print` works with a writer like Vec<u8>

        // Pipe output to `less`
        let mut pager = Command::new("less")
            .arg("-RS") // Preserve formatting & allow horizontal scrolling
            .stdin(Stdio::piped())
            .spawn()
            .expect("Failed to launch less");

        pager.stdin.unwrap().write_all(&output)?;
        Ok(())
    }
}
