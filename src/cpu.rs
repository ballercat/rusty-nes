// const ZERO_PAGE_TOP: usize = 0x100;
// const STACK_TOP: usize = 0x200;
const MEMORY_MAX: usize = 0x10000;
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
    }

    pub enum Mode {
        Immediate,
        Implied,
    }

    pub type Operation = (
        /* acc      */ u8,
        /* operand  */ u8,
        /* result   */ u8,
        /* flags    */ u8,
    );

    pub type Opcode = fn(State, u8) -> State;

    #[derive(Copy, Clone)]
    pub struct State {
        pub a: u8,
        pub pc: usize,
        pub x: u8,
        pub y: u8,
        pub status: u8,
        pub cycles: u32,
    }

    pub struct Processor {
        pub mem: Memory,
        pub state: State,
        pub cycles: u32,
    }

    /**
     * Calculate new Status flag based on the operation
     */
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

    pub fn adc(state: State, operand: u8) -> State {
        let carry = state.status & 1;
        let (mut result, ..) = state.a.overflowing_add(operand);
        result += carry;
        State {
            a: result,
            pc: state.pc + 2,
            status: nescpu::calc_status(
                state.status,
                (state.a, operand, result, N_FLAG | Z_FLAG | C_FLAG | V_FLAG),
            ),
            cycles: state.cycles + 2,
            ..state
        }
    }

    pub fn and(state: State, operand: u8) -> State {
        let result = state.a & operand;
        State {
            a: result,
            pc: state.pc + 2,
            status: nescpu::calc_status(
                state.status,
                (state.a, operand, result, N_FLAG | Z_FLAG),
            ),
            cycles: state.cycles + 2,
            ..state
        }
    }

    pub fn nop(state: State, _operand: u8) -> State {
        State {
            pc: state.pc + 1,
            cycles: state.cycles + 1,
            ..state
        }
    }

    impl Processor {
        pub fn new(mem: Option<Memory>) -> Processor {
            let state = State {
                a: 0,
                pc: 0,
                x: 0,
                y: 0,
                status: 0,
                cycles: 0,
            };
            Processor {
                mem: mem.unwrap_or(Memory::new()),
                state,
                cycles: 0,
            }
        }

        pub fn exec(&mut self) {
            let State { pc, .. } = self.state;
            let (opcode, mode) = self.decode(self.mem.read(pc));
            let operand = self.lookup(mode);

            self.state = opcode(self.state, operand);
        }

        pub fn lookup(&mut self, mode: Mode) -> u8 {
            match mode {
                Mode::Immediate => self.mem.read(self.state.pc + 1),
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
                    nescpu::and,
                    match b {
                        0..=7 => Mode::Immediate,
                        _ => Mode::Implied,
                    },
                ),
                3 => (
                    nescpu::adc,
                    match b {
                        0..=7 => Mode::Immediate,
                        _ => Mode::Implied,
                    },
                ),
                _ => (nescpu::nop, Mode::Implied),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use nescpu::Memory;
    use nescpu::State;

    #[test]
    fn test_memory() {
        let mut mem = Memory::new();
        mem.write(0, 24);

        assert_eq!(mem.read(0), 24);
        assert_eq!(mem.read(0x800), 24);
        assert_eq!(mem.read(0x800 * 2), 24);
        assert_eq!(mem.read(0x800 * 3), 24);
    }

    #[test]
    fn test_status_flags() {
        //  http://www.righto.com/2012/12/the-6502-overflow-flag-explained.html
        let overflow_table = [
            // "#0 No unsigned carry or signed overflow",
            (0x50, 0x10, 0x60, 0b0000_0000),
            // "#1 No unsigned carry but signed overflow",
            (0x50, 0x50, 0xae, 0b0100_0000),
            // "#2 No unsigned carry or signed overflow",
            (0x50, 0x90, 0xe0, 0b0000_0000),
            // "#3 Unsigned carry, but no signed overflow",
            (0x50, 0xd0, 0x120, 0b0000_0000),
            // "#4 No unsigned carry or signed overflow",
            (0xd0, 0x10, 0xe0, 0b0000_0000),
            // "#5 Unsigned carry but no signed overflow",
            (0xd0, 0x50, 0x120, 0b0000_0000),
            // "#6 Unsigned carry and signed overflow",
            (0xd0, 0x90, 0x160, 0b0100_0000),
            // "#7 Unsigned carry, but no signed overflow",
            (0xd0, 0xd0, 0x1a0, 0b0000_0000),
        ];

        for i in 0..overflow_table.len() {
            let (m, n, result, expected) = overflow_table[i];
            // cpu.set_status(m, n, result as u8, V_FLAG);
            let status = nescpu::calc_status(0, (m, n, result as u8, V_FLAG));
            assert_eq!(
                status, expected,
                "VFLAG m: {} n: {} result: {}",
                m, n, result
            );
        }
    }

    #[test]
    fn test_adc() {
        let base = State {
            pc: 0,
            a: 0,
            x: 0,
            y: 0,
            status: 0,
            cycles: 0,
        };
        // a + operand + carry_flag
        let state = nescpu::adc(
            State {
                a: 1,
                status: 0 | C_FLAG,
                ..base
            },
            1,
        );
        assert_eq!(state.a, 3);
        assert_eq!(state.pc, 2);
        assert_eq!(state.cycles, 2);

        let state = nescpu::adc(State { a: 1, ..base }, 0xFF);
        assert_eq!(state.a, 0);
        assert_eq!(state.status, Z_FLAG | C_FLAG);

        // This will check the overflow logic of 0xFF + 0xFF + 1 = 0xFF
        let state = nescpu::adc(
            State {
                a: 0xFF,
                status: C_FLAG,
                ..base
            },
            0xFF,
        );
        assert_eq!(state.a, 0xFF);
        assert_eq!(state.status, N_FLAG | C_FLAG);

        let state = nescpu::adc(State { a: 0x50, ..base }, 0x50);
        assert_eq!(state.a, 0xa0);
        assert_eq!(state.status, N_FLAG | V_FLAG);
    }

    #[test]
    fn test_and() {
        let base = State {
            pc: 0,
            a: 0,
            x: 0,
            y: 0,
            status: 0,
            cycles: 0,
        };
        let state = nescpu::and(State { a: 0b11, ..base }, 0b10);
        assert_eq!(state.a, 0b10);
        assert_eq!(state.cycles, 2);

        // Negative flag
        let state = nescpu::and(State { a: 0xff, ..base }, N_FLAG);
        assert_eq!(state.a, 0b1000_0000);
        assert_eq!(state.status, N_FLAG);

        // Zero flag
        let state = nescpu::and(State { a: 0xff, ..base }, 0);
        assert_eq!(state.a, 0);
        assert_eq!(state.status, Z_FLAG);
    }
}
