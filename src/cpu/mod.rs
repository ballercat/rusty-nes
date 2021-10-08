mod addressing;
pub mod base;
mod memory;
mod opcodes;

use base::Processor;
use memory::RESET_VECTOR;
use opcodes::encode;

impl Processor {
    pub fn reset(&mut self) {
        let lower = self.mem.read(RESET_VECTOR) as usize;
        let upper = self.mem.read(RESET_VECTOR + 1) as usize;
        self.state.pc = lower | (upper << 8);

        self.state.sp = 0xff;
    }

    pub fn exec(&mut self) {
        let value = self.mem.read(self.state.pc);
        let (opcode, mode) = self.decode(value);
        println!("{:#04x}: {:#04x}", self.state.pc, value);
        opcode(self, mode);
    }

    pub fn run_program(&mut self, program: &String) -> Vec<u8> {
        let lines = program.trim().lines();
        let mut result: Vec<u8> = Vec::new();
        for line in lines {
            result.append(&mut encode(&String::from(line.trim())));
        }
        // encode(program);
        result
    }
}

#[cfg(test)]
mod test {
    use super::base::N_FLAG;
    use super::memory::ROM_START;
    // use super::opcodes::{
    //     BCC, BCS, BEQ, BIT_A, BIT_Z, BMI, BNE, BPL, CLC, LDA, NOP, SEC,
    // };

    use super::*;

    fn run_program(cpu: &mut Processor, text: &String) {
        let lines = text.trim().lines();
        let mut program: Vec<u8> = Vec::new();
        for line in lines {
            program.append(&mut encode(&String::from(line.trim())));
        }

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
        //        run_cpu(&mut cpu, vec![LDA, 0x01, 0x38, 0x69, 0x01]);
        run_program(
            &mut cpu,
            &String::from(
                "
            LDA #$01;
            SEC     ;
            ADC #$01;
        ",
            ),
        );
        // a + operand + carry_flag
        assert_eq!(cpu.state.a, 3, "ADC result should be {}", 3);
    }

    #[test]
    fn test_and() {
        let mut cpu = Processor::new(None);
        // LDA# 03
        // AND# 02
        run_program(
            &mut cpu,
            &String::from(
                "
        LDA #$03;
        AND #$02;",
            ),
        );

        assert_eq!(cpu.state.a, 0b10, "AND result should be {}", 0b10);
    }

    #[test]
    fn test_asl() {
        let mut cpu = Processor::new(None);
        // LDA# 01
        // ASL A
        // run_cpu(&mut cpu, vec![0xa9, 0x02, 0x0A]);
        run_program(
            &mut cpu,
            &String::from(
                "
        LDA #$02;
        ASL A;
        ",
            ),
        );
        assert_eq!(cpu.state.a, 4, "ASL A result should be {}", 4);
    }

    #[test]
    fn test_branches() {
        let mut cpu = Processor::new(None);
        // Jump over NOP because of carry, jump back to start because of carry clear
        let program = String::from(
            "
        SEC     ; set accumulator
        BCS !$03; brach foward +3 because accumulator is set
        NOP     ; this should be skipped
        CLC     ;
        BCC !$FB; branch to start because accumulator is clear
        ",
        );
        run_program(&mut cpu, &program);
        assert_eq!(
            cpu.state.pc, ROM_START,
            "Branch BCS and reverse branch with BCC"
        );

        let program = String::from(
            "
        LDA #$00;
        BEQ !$FE;
        ",
        );
        run_program(&mut cpu, &program);
        assert_eq!(cpu.state.pc, ROM_START, "Branch via BEQ");

        // // Testing BIT as well as BMI below

        cpu.mem.write(0xFF, N_FLAG); // write to zer-page address 0xff
        let program = String::from(
            "
        BIT $FF  ; bit test with value using zero-page
        BMI !$FE ; branch
       ",
        );
        run_program(&mut cpu, &program);
        assert_eq!(cpu.state.pc, ROM_START, "Branch via BMI");

        let program = String::from(
            "
        BIT $FF00; $LLHH low & high bytes are swapped in memory
        BMI !$FD ;
        ",
        );
        run_program(&mut cpu, &program);
        assert_eq!(cpu.state.pc, ROM_START, "Branch via BMI");

        let program = String::from(
            "
        LDA #$01;
        BNE !$FE;
        ",
        );
        run_program(&mut cpu, &program);
        assert_eq!(cpu.state.pc, ROM_START, "Branch via BNE");

        let program = String::from(
            "
        LDA #$01;
        BPL !$FE;
        ",
        );
        run_program(&mut cpu, &program);
        assert_eq!(cpu.state.pc, ROM_START, "Branch via BPL");
    }
}
