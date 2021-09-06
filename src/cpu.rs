// const ZERO_PAGE_TOP: usize = 0x100;
// const STACK_TOP: usize = 0x200;
const RAM_TOP: usize = 0x800;
const MIRROR_TOP: usize = 0x2000;
const SIGN_BIT: u8 = 0b1000_0000;

const N_FLAG: u8 = 0b1000_0000;
const V_FLAG: u8 = 0b0100_0000;
// const B_FLAG: u8 = 0b0001_0000;
// const D_FLAG: u8 = 0b0000_1000;
// const I_FLAG: u8 = 0b0000_0100;
const Z_FLAG: u8 = 0b0000_0010;
const C_FLAG: u8 = 0b0000_0001;

pub mod nescpu {
    use super::*;

    pub struct Processor {
        pub mem: [u8; 0x10000],
        pub a: u8,
        pub pc: usize,
        pub x: u8,
        pub y: u8,
        pub status: u8,
        pub cycles: u32,
    }

    impl Processor {
        pub fn new() -> Processor {
            Processor {
                // TODO: DI memory
                mem: [0; 0x10000],
                a: 0,
                pc: 0,
                x: 0,
                y: 0,
                status: 0,
                cycles: 0,
            }
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

        pub fn set_status(&mut self, m: u8, n: u8, result: u8) {
            self.set_status_flag(C_FLAG, m as u16 + n as u16 > 0xFF);
            self.set_status_flag(Z_FLAG, result == 0);
            self.set_status_flag(N_FLAG, result & SIGN_BIT != 0);

            // Overflow logic is a bit more complicated

            // XOR-ing m & n is going to clear the SIGN_BIT if it's not == in BOTH
            let operands_match = ((m ^ n) & SIGN_BIT) == 0;
            let result_operands_match = ((m ^ result) & SIGN_BIT) == 0;
            let overflow = operands_match && !result_operands_match;

            self.set_status_flag(V_FLAG, overflow);
        }

        pub fn set_status_flag(&mut self, flag: u8, value: bool) {
            if value {
                self.status |= flag
            } else {
                self.status &= !flag
            }
        }

        pub fn get_status_flag(&self, flag: u8) -> bool {
            self.status & flag != 0
        }

        pub fn exec(&mut self) {
            let opcode = self.mem[self.pc];
            match opcode {
                0x69 => {
                    let operand = self.mem[self.pc + 1];
                    let carry = self.status & 1;
                    let result = (self.a + operand + carry) & 0xFF;

                    self.set_status(self.a, operand, result);
                }
                _ => {}
            }
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
        assert_eq!(cpu.read(0x800 * 2), 24);
        assert_eq!(cpu.read(0x800 * 3), 24);
    }

    #[test]
    fn test_overflow_flag() {
        let mut cpu = Processor::new();

        //  http://www.righto.com/2012/12/the-6502-overflow-flag-explained.html
        let overflow_table = [
            (
                0x50,
                0x10,
                0x60,
                false,
                "#0 No unsigned carry or signed overflow",
            ),
            (
                0x50,
                0x50,
                0xae,
                true,
                "#1 No unsigned carry but signed overflow",
            ),
            (
                0x50,
                0x90,
                0xe0,
                false,
                "#2 No unsigned carry or signed overflow",
            ),
            (
                0x50,
                0xd0,
                0x120,
                false,
                "#3 Unsigned carry, but no signed overflow",
            ),
            (
                0xd0,
                0x10,
                0xe0,
                false,
                "#4 No unsigned carry or signed overflow",
            ),
            (
                0xd0,
                0x50,
                0x120,
                false,
                "#5 Unsigned carry but no signed overflow",
            ),
            (
                0xd0,
                0x90,
                0x160,
                true,
                "#6 Unsigned carry and signed overflow",
            ),
            (
                0xd0,
                0xd0,
                0x1a0,
                false,
                "#7 Unsigned carry, but no signed overflow",
            ),
        ];

        for i in 0..overflow_table.len() {
            let (m, n, result, expected, description) = overflow_table[i];
            cpu.set_status(m, n, result as u8);
            assert_eq!(
                cpu.get_status_flag(V_FLAG),
                expected,
                "{}",
                description
            );
        }
    }
}
