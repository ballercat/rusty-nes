mod base;
mod memory;
mod opcodes;

use base::{Mode, Processor, State};
use memory::{Memory, RESET_VECTOR};

pub mod nescpu {
    use super::*;

    pub type Operation = (
        /* acc      */ u8,
        /* operand  */ u8,
        /* result   */ u8,
        /* flags    */ u8,
    );

    pub type Opcode = fn(&mut Processor, Mode) -> ();

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
    }
}

#[cfg(test)]
mod test {
    use super::base::V_FLAG;
    use super::memory::ROM_START;
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
