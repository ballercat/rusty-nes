mod addressing;
mod base;
mod memory;
mod opcodes;

use addressing::Mode;
use base::{Processor, State};
use memory::RESET_VECTOR;

pub mod nescpu {
    use super::*;

    pub type Opcode = fn(&mut Processor, Mode) -> ();

    impl Processor {
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
    use super::memory::ROM_START;
    use super::*;

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
