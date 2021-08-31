// const ZERO_PAGE_TOP: usize = 0x100;
// const STACK_TOP: usize = 0x200;
const RAM_TOP: usize = 0x800;
const MIRROR_TOP: usize = 0x2000;

pub mod nescpu {
    use super::*;

    pub struct Processor {
        pub mem: [u8;0x10000],
    }
    pub fn power() -> Processor {
        Processor {
            mem: [0;0x10000],
        }
    }

    pub fn write(cpu: &mut Processor, address: usize, value: u8) {
        cpu.mem[address] = value;

        // Internal RAM mirrors
        if address <= RAM_TOP {
            cpu.mem[address + 0x800] = value;
            // cpu.mem[address + 0x800*2] = value;
            // cpu.mem[address + 0x800*3] = value;
        }

        // if address <= 0x2007 && adress >= 0x2000 {
        // }
    }

    pub fn read(cpu: &mut Processor, address: usize) -> u8 {
        if address < MIRROR_TOP {
            return cpu.mem[address % RAM_TOP];
        }

        cpu.mem[address]
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_power() {
        let cpu = nescpu::power();
        assert_eq!(cpu.mem.len(), 0x10000);
    }

    #[test]
    fn test_read_write() {
        let mut cpu = nescpu::power();
        nescpu::write(&mut cpu, 0, 24);

        assert_eq!(nescpu::read(&mut cpu, 0), 24);
        assert_eq!(nescpu::read(&mut cpu,0x800), 24);
        assert_eq!(nescpu::read(&mut cpu,0x800*2), 24);
        assert_eq!(nescpu::read(&mut cpu,0x800*3), 24);
    }
}
