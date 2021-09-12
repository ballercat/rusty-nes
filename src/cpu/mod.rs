mod addressing;
pub mod base;
mod memory;
mod opcodes;

use base::Processor;
use memory::RESET_VECTOR;

impl Processor {
    pub fn reset(&mut self) {
        let lower = self.mem.read(RESET_VECTOR) as usize;
        let upper = self.mem.read(RESET_VECTOR + 1) as usize;
        self.state.pc = lower | (upper << 8);
    }

    pub fn exec(&mut self) {
        let value = self.mem.read(self.state.pc);
        let (opcode, mode) = self.decode(value);
        println!("{:#04x}: {:#04x}", self.state.pc, value);
        opcode(self, mode);
    }
}

#[cfg(test)]
mod test {
    use super::base::N_FLAG;
    use super::memory::ROM_START;
    use super::opcodes::{
        BCC, BCS, BEQ, BIT_A, BIT_Z, BMI, BNE, CLC, LDA, NOP, SEC,
    };

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

            // terminate on loops
            if cpu.state.pc < old_pc {
                break;
            }

            // terminate when we run out of instructions
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
        run_cpu(&mut cpu, vec![LDA, 0x01, 0x38, 0x69, 0x01]);

        // a + operand + carry_flag
        assert_eq!(cpu.state.a, 3, "ADC result should be {}", 3);
    }

    #[test]
    fn test_and() {
        let mut cpu = Processor::new(None);
        // LDA# 03
        // AND# 02
        run_cpu(&mut cpu, vec![LDA, 0b11, 0x29, 0b10]);
        assert_eq!(cpu.state.a, 0b10, "AND result should be {}", 0b10);
    }

    #[test]
    fn test_asl() {
        let mut cpu = Processor::new(None);
        // LDA# 01
        // ASL A
        run_cpu(&mut cpu, vec![0xa9, 0x02, 0x0A]);
        assert_eq!(cpu.state.a, 4, "ASL A result should be {}", 4);
    }

    #[test]
    fn test_branches() {
        let mut cpu = Processor::new(None);
        // Jump over NOP because of carry, jump back to start because of carry clear
        run_cpu(&mut cpu, vec![SEC, BCS, 0x03, NOP, CLC, BCC, 0xfb]);
        assert_eq!(
            cpu.state.pc, ROM_START,
            "Branch BCS and reverse branch with BCC"
        );

        run_cpu(&mut cpu, vec![LDA, 0, BEQ, 0xfe]);
        assert_eq!(cpu.state.pc, ROM_START, "Branch via BEQ");

        cpu.mem.write(0xFF, N_FLAG);
        // Testing BIT as well as BMI below
        run_cpu(&mut cpu, vec![BIT_Z, 0xFF, BMI, 0xfe]);
        assert_eq!(cpu.state.pc, ROM_START, "Branch via BMI");

        run_cpu(&mut cpu, vec![BIT_A, 0xFF, 0x00, BMI, 0xfd]);
        assert_eq!(cpu.state.pc, ROM_START, "Branch via BMI");

        run_cpu(&mut cpu, vec![LDA, 0x01, BNE, 0xfe]);
        assert_eq!(cpu.state.pc, ROM_START, "Branch via BNE");
    }
}
