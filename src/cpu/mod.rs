// const ZERO_PAGE_TOP: usize = 0x100;
// const STACK_TOP: usize = 0x200;
const RESET_VECTOR: usize = 0xFFFC;
const SIGN_BIT: u8 = 0b1000_0000;

const N_FLAG: u8 = 0b1000_0000;
const V_FLAG: u8 = 0b0100_0000;
// const B_FLAG: u8 = 0b0001_0000;
// const D_FLAG: u8 = 0b0000_1000;
// const I_FLAG: u8 = 0b0000_0100;
const Z_FLAG: u8 = 0b0000_0010;
const C_FLAG: u8 = 0b0000_0001;

mod memory;
use memory::Memory;

pub mod nescpu {
    use super::*;

    pub enum Mode {
        Immediate,
        Implied,
    }

    pub enum Reg {
        A,
        X,
        Y,
        S,
    }

    pub type Operation = (
        /* acc      */ u8,
        /* operand  */ u8,
        /* result   */ u8,
        /* flags    */ u8,
    );

    pub type Opcode = fn(&mut Processor, Mode) -> ();

    #[derive(Copy, Clone)]
    pub struct State {
        pub a: u8,
        pub pc: usize,
        pub x: u8,
        pub y: u8,
        pub status: u8,
    }

    pub struct Processor {
        pub mem: Memory,
        pub state: State,
        pub cycles: u32,
    }

    impl Processor {
        pub fn new(mem: Option<Memory>) -> Processor {
            let state = State {
                a: 0,
                pc: 0,
                x: 0,
                y: 0,
                status: 0,
            };
            Processor {
                mem: mem.unwrap_or(Memory::new()),
                state,
                cycles: 0,
            }
        }

        pub fn reset(&mut self) {
            let lower = self.mem.read(RESET_VECTOR) as usize;
            let upper = self.mem.read(RESET_VECTOR + 1) as usize;
            self.state.pc = lower | (upper << 8);
        }

        pub fn exec(&mut self) {
            let State { pc, .. } = self.state;
            let (opcode, mode) = self.decode(self.mem.read(pc));
            println!("Decode pc {}", pc);
            opcode(self, mode);
        }

        pub fn lookup(&mut self, mode: Mode) -> u8 {
            match mode {
                Mode::Immediate => {
                    println!("Lookup Immediate value at {}", self.state.pc + 1);
                    self.mem.read(self.state.pc + 1)
                }
                Mode::Implied => {
                    self.cycles += 1;
                    0
                }
            }
        }

        pub fn decode(&self, value: u8) -> (Opcode, Mode) {
            // https://www.masswerk.at/6502/6502_instruction_set.html#layout
            let a = (value & 0b1110_0000) >> 5;
            let b = (value & 0b0001_1100) >> 2;
            let c = value & 0b0000_0011;

            match (c, b, a) {
                (0, 6, 1) => (Processor::sec, Mode::Implied),
                (1, 2, 1) => (Processor::and, Mode::Immediate),
                (1, 2, 3) => (Processor::adc, Mode::Immediate),
                (1, 2, 5) => (Processor::lda, Mode::Immediate),
                _ => (Processor::nop, Mode::Implied),
            }
        }

        pub fn get_pc(&self) -> usize {
            self.state.pc
        }

        pub fn update_pc(&mut self, delta: i32) -> &mut Self {
            println!("Update pc {} with {}", self.state.pc, delta);
            if delta.is_negative() {
                self.state.pc -= delta.wrapping_abs() as u32 as usize;
            } else {
                self.state.pc += delta as usize;
            }
            self
        }

        pub fn get_reg(&self, reg: Reg) -> u8 {
            match reg {
                Reg::X => self.state.x,
                Reg::Y => self.state.y,
                Reg::A => self.state.a,
                Reg::S => self.state.status,
            }
        }

        pub fn set_reg(&mut self, reg: Reg, value: u8) -> &mut Self {
            match reg {
                Reg::X => self.state.x = value,
                Reg::Y => self.state.y = value,
                Reg::A => self.state.a = value,
                Reg::S => self.state.status = value,
            };
            self
        }

        pub fn update_cycles(&mut self, cycles: u32) -> &mut Self {
            self.cycles += cycles;
            self
        }

        /**
         * Calculate new Status flag based on the operation
         */
        pub fn update_status(
            &mut self,
            m: u8,
            n: u8,
            result: u8,
            flags: u8,
        ) -> &mut Self {
            let mut new_status = self.get_reg(Reg::S);
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

            self.set_reg(Reg::S, new_status);

            self
        }

        pub fn adc(&mut self, mode: Mode) {
            let operand = self.lookup(mode);
            let accumulator = self.state.a;
            let carry = self.state.status & 1;
            println!(
                "operand {} accumulator {} carry {}",
                operand, accumulator, carry
            );
            let (mut result, ..) = accumulator.overflowing_add(operand);
            result += carry;
            self.set_reg(Reg::A, result)
                .update_pc(2)
                .update_status(
                    accumulator,
                    operand,
                    result,
                    N_FLAG | Z_FLAG | C_FLAG | V_FLAG,
                )
                .update_cycles(2);
        }

        pub fn and(&mut self, mode: Mode) {
            let operand = self.lookup(mode);
            let accumulator = self.get_reg(Reg::A);
            let result = accumulator & operand;
            self.set_reg(Reg::A, result)
                .update_pc(2)
                .update_status(accumulator, operand, result, N_FLAG | Z_FLAG)
                .update_cycles(2);
        }
        pub fn asl(&mut self, mode: Mode) {
            let operand = self.lookup(mode);
            let result = operand << 1;
            let accumulator = self.get_reg(Reg::A);
            self.set_reg(Reg::A, result)
                .update_status(
                    accumulator,
                    operand,
                    result,
                    Z_FLAG | C_FLAG | N_FLAG,
                )
                .update_cycles(2);
        }

        pub fn lda(&mut self, mode: Mode) {
            let operand = self.lookup(mode);
            println!("Load accumulator with {}", operand);
            self.set_reg(Reg::A, operand)
                .update_pc(2)
                .update_status(operand, operand, operand, Z_FLAG | N_FLAG)
                .update_cycles(2);
        }

        pub fn sec(&mut self, _mode: Mode) {
            println!("Set carry flag");
            self.state.status |= C_FLAG;
            self.update_pc(1).update_cycles(2);
        }

        pub fn nop(&mut self, _mode: Mode) {
            println!("NOP");
            self.update_pc(1).update_cycles(1);
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use nescpu::Processor;

    const ROM_START: usize = 0x8000;

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

        let mut cpu = Processor::new(None);

        for i in 0..overflow_table.len() {
            let (m, n, result, expected) = overflow_table[i];
            cpu.update_status(m, n, result as u8, V_FLAG);
            assert_eq!(
                cpu.state.status, expected,
                "VFLAG m: {} n: {} result: {}",
                m, n, result
            );
        }
    }

    fn run_cpu(cpu: &mut Processor, program: Vec<u8>) {
        let reset_vector =
            [(ROM_START & 0xFF) as u8, ((ROM_START & 0xFF00) >> 8) as u8];

        // Load the program into memory
        cpu.mem.load(ROM_START, &program);
        // Setup reset vector to start PC at ROM_START
        cpu.mem.load(RESET_VECTOR, &reset_vector);

        cpu.reset();

        loop {
            let old_pc = cpu.state.pc;
            cpu.exec();

            if old_pc == cpu.state.pc {
                panic!("Program counter did not update, force quitting!");
            }

            if cpu.state.pc - ROM_START >= program.len() {
                break;
            }
        }
    }

    #[test]
    fn test_adc() {
        let mut cpu = Processor::new(None);
        // LDA# 01 ; load accumulator
        // SEC     ; set carry flag
        // ADC# 01 ; add with carry
        run_cpu(&mut cpu, vec![0xa9, 0x01, 0x38, 0x69, 0x01]);

        // a + operand + carry_flag
        assert_eq!(cpu.state.a, 3, "ADC result should be {}", 3);
    }

    #[test]
    fn test_and() {
        let mut cpu = Processor::new(None);
        // LDA# 03
        // AND# 02
        run_cpu(&mut cpu, vec![0xa9, 0b11, 0x29, 0b10]);
        assert_eq!(cpu.state.a, 0b10, "AND result should be {}", 0b10);
    }
}
