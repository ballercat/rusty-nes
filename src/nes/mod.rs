use super::cpu::base::Processor;
use super::cpu::memory::{RESET_VECTOR, ROM_START};
use std::iter::FromIterator;

const KILOBYTE_BYTE_SIZE: usize = 1024;
const PRG_ROM_UNIT_SIZE: usize = KILOBYTE_BYTE_SIZE * 16;
const HEADER_BYTE_SIZE: usize = 16;
const TRAINER_BYTE_SIZE: usize = KILOBYTE_BYTE_SIZE / 2;

#[derive(Debug)]
pub struct Cartridge {
    pub header: String,
    pub rom: Vec<u8>,
}

impl Cartridge {
    pub fn new() -> Cartridge {
        Cartridge {
            header: String::from("empty"),
            rom: Vec::new(),
        }
    }

    pub fn load(&mut self, data: &Vec<u8>) {
        self.header = String::from_utf8_lossy(&data[0..3]).into_owned();
        let control_byte_1 = data[6];
        //  let vrom_size = data[5] as usize * KILOBYTE_BYTE_SIZE * 8;
        let rom_start = HEADER_BYTE_SIZE
            + ((control_byte_1 as usize & 0b0000_0100) / 0b0000_0100
                * TRAINER_BYTE_SIZE);
        let rom_size = data[4] as usize * PRG_ROM_UNIT_SIZE;
        let rom_end = rom_start + rom_size;

        self.rom = Vec::from_iter(data[rom_start..rom_end].iter().cloned());
    }
}

#[derive(Debug)]
pub struct Nes {
    pub cartridge: Cartridge,
    pub cpu: Processor,
}

impl Nes {
    pub fn new(cpu: Processor) -> Nes {
        Nes {
            cpu: cpu,
            cartridge: Cartridge::new(),
        }
    }
    pub fn load_cartridge(&mut self, filename: &String) {
        let data = match std::fs::read(filename) {
            Ok(bytes) => bytes,
            Err(e) => {
                if e.kind() == std::io::ErrorKind::PermissionDenied {
                    eprintln!(
                        "Permission denied while attmepting to read .nes file."
                    );
                    return;
                }

                panic!("{}", e);
            }
        };

        self.cartridge.load(&data);
    }

    pub fn reset(&mut self, reset_pc: Option<usize>) {
        let rom = &self.cartridge.rom;

        let reset_vector = [
            (reset_pc.unwrap_or(ROM_START) & 0xFF) as u8,
            ((reset_pc.unwrap_or(ROM_START) & 0xFF00) >> 8) as u8,
        ];

        // Load the program into memory
        self.cpu.mem.load(ROM_START, &rom);
        if rom.len() <= PRG_ROM_UNIT_SIZE {
            // Any cartridge with under 16K ROM should load both into 0x8000 and 0xC000
            self.cpu.mem.load(ROM_START + PRG_ROM_UNIT_SIZE, &rom);
        }
        // Setup reset vector to start PC at ROM_START
        self.cpu.mem.load(RESET_VECTOR, &reset_vector);

        self.cpu.reset();
    }

    pub fn run(&mut self) {
        let mut limit = 10000;
        loop {
            self.cpu.exec();
            limit -= 1;
            if limit < 0 {
                break;
            }
        }
        println!("STOP NES");
    }
}
