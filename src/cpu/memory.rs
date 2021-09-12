pub const MEMORY_MAX: usize = 0x10000;
pub const RAM_TOP: usize = 0x800;
pub const MIRROR_TOP: usize = 0x2000;
#[allow(dead_code)]
pub const ZERO_PAGE_TOP: usize = 0x100;
#[allow(dead_code)]
pub const STACK_TOP: usize = 0x200;
#[allow(dead_code)]
pub const RESET_VECTOR: usize = 0xFFFC;
#[allow(dead_code)]
pub const ROM_START: usize = 0x8000;

pub struct Memory {
    ram: [u8; MEMORY_MAX],
}

impl Memory {
    pub fn new() -> Memory {
        Memory {
            ram: [0; MEMORY_MAX],
        }
    }

    pub fn write(&mut self, address: usize, value: u8) {
        self.ram[address] = value;
    }

    pub fn read(&self, address: usize) -> u8 {
        if address < MIRROR_TOP {
            return self.ram[address % RAM_TOP];
        }
        self.ram[address]
    }

    pub fn load(&mut self, address: usize, data: &[u8]) {
        self.ram[address..address + data.len()].copy_from_slice(data);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_memory() {
        let mut mem = Memory::new();
        mem.write(0, 24);

        assert_eq!(mem.read(0), 24);
        assert_eq!(mem.read(0x800), 24);
        assert_eq!(mem.read(0x800 * 2), 24);
        assert_eq!(mem.read(0x800 * 3), 24);
    }
}
