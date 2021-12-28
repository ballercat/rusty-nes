mod addressing;
pub mod base;
pub mod memory;
mod opcodes;

use base::Processor;
use memory::{RESET_VECTOR, ROM_START};
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
        // let start = self.state.pc;
        // let end = start + opcode_len(mode) as usize;
        // let full = &self.mem.ram[start..end];
        println!("{:#04x}: {:#04x}", self.state.pc, value);
        opcode(self, mode);
    }

    pub fn run_program(&mut self, text: &String) {
        let lines = text.trim().lines();
        let mut program: Vec<u8> = Vec::new();
        for line in lines {
            program.append(&mut encode(&String::from(line.trim())));
        }

        let reset_vector =
            [(ROM_START & 0xFF) as u8, ((ROM_START & 0xFF00) >> 8) as u8];

        // Load the program into memory
        self.mem.load(ROM_START, &program);
        // Setup reset vector to start PC at ROM_START
        self.mem.load(RESET_VECTOR, &reset_vector);

        self.reset();

        loop {
            let old_pc = self.state.pc;
            let value = self.mem.read(self.state.pc);
            // 0x00/Zero opcode is the BRK instruction
            if value == 0x00 {
                println!("Encountered BRK. Exiting.");
                break;
            }
            let (opcode, mode) = self.decode(value);
            opcode(self, mode);

            if old_pc == self.state.pc {
                panic!("Program counter did not update, force quitting!");
            }

            // terminate on loops
            if self.state.pc < old_pc {
                break;
            }

            // terminate when we run out of instructions
            if self.state.pc - ROM_START >= program.len() {
                break;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::memory::ROM_START;
    use super::*;

    #[test]
    fn test_math() {
        let mut cpu = Processor::new(None);
        cpu.run_program(&String::from(
            "
            LDA #$01; load accumulator
            SEC     ; set carry flag
            ADC #$01; add with carry
        ",
        ));
        // a + operand + carry_flag
        assert_eq!(cpu.state.a, 3, "ADC result should be {}", 3);

        cpu.run_program(&String::from(
            "
        LDA #$03;
        AND #$02;",
        ));

        assert_eq!(cpu.state.a, 0b10, "AND result should be {}", 0b10);

        cpu.run_program(&String::from(
            "
        LDA #$02;
        ASL A;
        ",
        ));
        assert_eq!(cpu.state.a, 4, "ASL A result should be {}", 4);
    }

    #[test]
    fn test_branches() {
        let mut cpu = Processor::new(None);
        cpu.run_program(&String::from(
            "
        SEC     ; set accumulator
        BCS !$03; brach foward +3 because accumulator is set
        NOP     ; this should be skipped
        CLC     ; carry clear should cause the next instruction to jump back
        BCC !$FB; branch to start because accumulator is clear
        ",
        ));
        assert_eq!(
            cpu.state.pc, ROM_START,
            "Branch BCS and reverse branch with BCC"
        );

        cpu.run_program(&String::from(
            "
        LDA #$00;
        BEQ !$FE;
        ",
        ));
        assert_eq!(cpu.state.pc, ROM_START, "Branch via BEQ");

        // Testing BIT as well as BMI below
        cpu.run_program(&String::from(
            "
        LDA #$80;
        STA $FF ;
        BIT $FF ; bit test with value using zero-page
        BMI !$FA; branch -6
       ",
        ));
        assert_eq!(cpu.state.pc, ROM_START, "Branch via BMI");

        cpu.run_program(&String::from(
            "
        BIT $FF00; $LLHH low & high bytes are swapped in memory
        BMI !$FD ;
        ",
        ));
        assert_eq!(cpu.state.pc, ROM_START, "Branch via BMI");

        cpu.run_program(&String::from(
            "
        LDA #$01;
        BNE !$FE;
        ",
        ));
        assert_eq!(cpu.state.pc, ROM_START, "Branch via BNE");

        cpu.run_program(&String::from(
            "
        LDA #$01;
        BPL !$FE;
        ",
        ));
        assert_eq!(cpu.state.pc, ROM_START, "Branch via BPL");
    }

    #[test]
    fn test_cld() {
        let mut cpu = Processor::new(None);
        cpu.run_program(&String::from(
            "
        LDA #$08; load bit 4 into A. This equals Decimal flag
        STA $FF; save it at address 0xFF
        SED    ;
        CLD    ;
        BRK    ; force exit
        ",
        ));

        assert_eq!(cpu.state.status, 0);
    }
}
