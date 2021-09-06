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

    pub enum Opcode {
        ADC,
        AND,
        NOP,
    }

    pub enum Mode {
        Immediate,
        Implied,
    }

    pub type Operation = (
        /*acc */ u8,
        /* operand */ u8,
        /* result */ u8,
        /* flags */ u8,
    );

    pub struct State {
        pub a: u8,
        pub pc: usize,
        pub x: u8,
        pub y: u8,
        pub status: u8,
        pub cycles: u32,
    }

    pub struct Processor {
        pub mem: [u8; 0x10000],
        pub state: State,
        pub cycles: u32,
    }

    pub fn calc_status(status: u8, op: Operation) -> u8 {
        let (m, n, result, flags) = op;
        let mut new_status = status;
        let mut merge_status = |flag: u8, value: bool| {
            if value {
                new_status |= flag
            } else {
                new_status &= !flag
            }
        };

        if flags & C_FLAG != 0 {
            merge_status(C_FLAG, m as u16 + n as u16 > 0xFF);
        }

        if flags & Z_FLAG != 0 {
            merge_status(Z_FLAG, result == 0);
        }

        if flags & N_FLAG != 0 {
            merge_status(N_FLAG, result & SIGN_BIT != 0);
        }

        // Overflow logic is a bit more complicated

        // XOR-ing m & n is going to clear the SIGN_BIT if it's not == in BOTH
        if flags & V_FLAG != 0 {
            let operands_match = ((m ^ n) & SIGN_BIT) == 0;
            let result_operands_match = ((m ^ result) & SIGN_BIT) == 0;
            let overflow = operands_match && !result_operands_match;

            merge_status(V_FLAG, overflow);
        }

        new_status
    }

    impl Processor {
        pub fn new() -> Processor {
            let state = State {
                a: 0,
                pc: 0,
                x: 0,
                y: 0,
                status: 0,
                cycles: 0,
            };
            Processor {
                // TODO: DI memory
                mem: [0; 0x10000],
                state: state,
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

        pub fn exec(&mut self) {
            let State { pc, .. } = self.state;
            let (opcode, mode) = self.decode(self.mem[pc]);
            match opcode {
                Opcode::ADC => {
                    let State { a, status, .. } = self.state;
                    let operand = self.lookup(mode);
                    let carry = status & 1;
                    let result = (a + operand + carry) & 0xFF;

                    self.state.cycles += 2;
                    self.state.status = nescpu::calc_status(
                        status,
                        (a, operand, result, N_FLAG | Z_FLAG | C_FLAG | V_FLAG),
                    );
                    self.state.a = result;
                }
                Opcode::AND => {
                    let State { a, status, .. } = self.state;
                    let operand = self.lookup(mode);
                    let result = a & operand;
                    self.state.cycles += 2;
                    self.state.status = nescpu::calc_status(
                        status,
                        (a, operand, result, N_FLAG | Z_FLAG),
                    );
                    self.state.a = result;
                }
                _ => {}
            }
        }

        pub fn lookup(&mut self, mode: Mode) -> u8 {
            match mode {
                Mode::Immediate => self.mem[self.state.pc + 1],
                Mode::Implied => {
                    self.state.cycles += 1;
                    0
                }
            }
        }

        pub fn decode(&self, value: u8) -> (Opcode, Mode) {
            // https://www.masswerk.at/6502/6502_instruction_set.html#layout
            let a = (value & 0b1110_0000) >> 5;
            let b = (value & 0b0001_1100) >> 2;
            let _c = value & 0b0000_0011;

            match a {
                1 => (
                    Opcode::AND,
                    match b {
                        0..=7 => Mode::Immediate,
                        _ => Mode::Implied,
                    },
                ),
                3 => (
                    Opcode::ADC,
                    match b {
                        0..=7 => Mode::Immediate,
                        _ => Mode::Implied,
                    },
                ),
                _ => (Opcode::NOP, Mode::Implied),
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
    fn test_status_flags() {
        //  http://www.righto.com/2012/12/the-6502-overflow-flag-explained.html
        let overflow_table = [
            (
                0x50,
                0x10,
                0x60,
                0b0000_0000,
                "#0 No unsigned carry or signed overflow",
            ),
            (
                0x50,
                0x50,
                0xae,
                0b0100_0000,
                "#1 No unsigned carry but signed overflow",
            ),
            (
                0x50,
                0x90,
                0xe0,
                0b0000_0000,
                "#2 No unsigned carry or signed overflow",
            ),
            (
                0x50,
                0xd0,
                0x120,
                0b0000_0000,
                "#3 Unsigned carry, but no signed overflow",
            ),
            (
                0xd0,
                0x10,
                0xe0,
                0b0000_0000,
                "#4 No unsigned carry or signed overflow",
            ),
            (
                0xd0,
                0x50,
                0x120,
                0b0000_0000,
                "#5 Unsigned carry but no signed overflow",
            ),
            (
                0xd0,
                0x90,
                0x160,
                0b0100_0000,
                "#6 Unsigned carry and signed overflow",
            ),
            (
                0xd0,
                0xd0,
                0x1a0,
                0b0000_0000,
                "#7 Unsigned carry, but no signed overflow",
            ),
        ];

        for i in 0..overflow_table.len() {
            let (m, n, result, expected, description) = overflow_table[i];
            // cpu.set_status(m, n, result as u8, V_FLAG);
            let status = nescpu::calc_status(0, (m, n, result as u8, V_FLAG));
            assert_eq!(status, expected, "{}", description);
        }
    }
}
