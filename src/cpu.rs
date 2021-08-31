// const ZERO_PAGE_TOP: usize = 0x100;
// const STACK_TOP: usize = 0x200;
const RAM_TOP: usize = 0x800;
const MIRROR_TOP: usize = 0x2000;

pub mod nescpu {
    use super::*;

    pub struct Processor {
        pub mem: [u8;0x10000],
    }

    impl Processor {
        pub fn new() -> Processor {
            Processor { mem: [0;0x10000], }
        }

        pub fn write(&mut self, address: usize, value: u8) {
            self.mem[address] = value;
        }

        pub fn read(&self, address: usize) -> u8 {
            if address < MIRROR_TOP {
                return self.mem[address % RAM_TOP];
            }
            self.mem[address]
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use nescpu::Processor;

    #[test]
    fn test_power() {
        let cpu = Processor::new();
        assert_eq!(cpu.mem.len(), 0x10000);
    }

    #[test]
    fn test_read_write() {
        let mut cpu = Processor::new();
        cpu.write(0, 24);

        assert_eq!(cpu.read(0), 24);
        assert_eq!(cpu.read(0x800), 24);
        assert_eq!(cpu.read(0x800*2), 24);
        assert_eq!(cpu.read(0x800*3), 24);
    }
}
