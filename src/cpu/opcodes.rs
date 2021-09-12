use super::addressing::Mode;
use super::base::{Processor, Reg, C_FLAG, N_FLAG, V_FLAG, Z_FLAG};

#[allow(dead_code)]
pub const ADC: u8 = 0x69;
#[allow(dead_code)]
pub const BCC: u8 = 0x90;
#[allow(dead_code)]
pub const BCS: u8 = 0xb0;
#[allow(dead_code)]
pub const BEQ: u8 = 0xf0;
#[allow(dead_code)]
pub const BIT_Z: u8 = 0x24;
#[allow(dead_code)]
pub const BIT_A: u8 = 0x2c;
#[allow(dead_code)]
pub const BMI: u8 = 0x30;
#[allow(dead_code)]
pub const BNE: u8 = 0xd0;
#[allow(dead_code)]
pub const BPL: u8 = 0x10;
#[allow(dead_code)]
pub const CLC: u8 = 0x18;
#[allow(dead_code)]
pub const SEC: u8 = 0x38;
#[allow(dead_code)]
pub const NOP: u8 = 0xea;
#[allow(dead_code)]
pub const LDA: u8 = 0xa9;

pub type Opcode = fn(&mut Processor, Mode) -> ();

pub fn opcode_len(mode: Mode) -> i32 {
    match mode {
        Mode::Absolute => 3,
        Mode::ZeroPage | Mode::Immediate => 2,
        _ => 1,
    }
}

impl Processor {
    pub fn decode(&self, value: u8) -> (Opcode, Mode) {
        // https://www.masswerk.at/6502/6502_instruction_set.html#layout
        let a = (value & 0b1110_0000) >> 5;
        let b = (value & 0b0001_1100) >> 2;
        let c = value & 0b0000_0011;

        match (c, b, a) {
            (0, 1, 1) => (Processor::bit, Mode::ZeroPage),
            (0, 3, 1) => (Processor::bit, Mode::Absolute),
            (0, 4, 0) => (Processor::bpl, Mode::Immediate),
            (0, 4, 1) => (Processor::bmi, Mode::Immediate),
            (0, 4, 4) => (Processor::bcc, Mode::Immediate),
            (0, 4, 5) => (Processor::bcs, Mode::Immediate),
            (0, 4, 6) => (Processor::bne, Mode::Immediate),
            (0, 4, 7) => (Processor::beq, Mode::Immediate),
            (0, 6, 0) => (Processor::clc, Mode::Implied),
            (0, 6, 1) => (Processor::sec, Mode::Implied),
            (1, 2, 1) => (Processor::and, Mode::Immediate),
            (1, 2, 3) => (Processor::adc, Mode::Immediate),
            (1, 2, 5) => (Processor::lda, Mode::Immediate),
            (2, 2, 0) => (Processor::asl, Mode::Accumulator),
            (2, 2, 7) => (Processor::nop, Mode::Implied),
            _ => (Processor::nop, Mode::Implied),
        }
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
            .update_pc(opcode_len(mode))
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
            .update_pc(opcode_len(mode))
            .update_status(accumulator, operand, result, N_FLAG | Z_FLAG)
            .update_cycles(2);
    }

    pub fn asl(&mut self, mode: Mode) {
        let operand = self.lookup(mode);
        let result = operand << 1;

        match mode {
            Mode::Accumulator => {
                self.set_reg(Reg::A, result);
            }
            _ => panic!("Unimplemented ASL addressing mode!"),
        };

        self.update_status(operand, 1, result, Z_FLAG | C_FLAG | N_FLAG)
            .update_pc(opcode_len(mode))
            .update_cycles(2);
    }

    pub fn bcc(&mut self, mode: Mode) {
        let operand = self.lookup(mode);
        let mut cycles = 2;
        if self.state.status & C_FLAG == 0 {
            cycles += 1;
            self.update_pc(operand as i8 as i32);
        } else {
            self.update_pc(opcode_len(mode));
        }

        self.update_cycles(cycles);
    }

    pub fn bcs(&mut self, mode: Mode) {
        let operand = self.lookup(mode);
        let mut cycles = 2;
        if self.state.status & C_FLAG != 0 {
            cycles += 1;
            self.update_pc(operand as i8 as i32);
        } else {
            self.update_pc(opcode_len(mode));
        }
        self.update_cycles(cycles);
    }

    pub fn beq(&mut self, mode: Mode) {
        let operand = self.lookup(mode);
        let mut cycles = 2;
        if self.state.status & Z_FLAG != 0 {
            cycles += 1;
            self.update_pc(operand as i8 as i32);
        } else {
            self.update_pc(opcode_len(mode));
        }
        self.update_cycles(cycles);
    }

    pub fn bit(&mut self, mode: Mode) {
        let operand = self.lookup(mode);
        let accumulator = self.state.a;
        let result = operand & accumulator;

        let new_flags = operand & (N_FLAG | V_FLAG);
        self.state.status =
            (self.state.status & !(N_FLAG | V_FLAG)) | new_flags;

        self.update_status(accumulator, operand, result, Z_FLAG)
            .update_pc(opcode_len(mode));
    }

    pub fn bmi(&mut self, mode: Mode) {
        let operand = self.lookup(mode);
        let mut cycles = 2;
        if self.state.status & N_FLAG != 0 {
            cycles += 1;
            self.update_pc(operand as i8 as i32);
        } else {
            self.update_pc(opcode_len(mode));
        }
        self.update_cycles(cycles);
    }

    pub fn bne(&mut self, mode: Mode) {
        let operand = self.lookup(mode);
        let mut cycles = 2;
        if self.state.status & Z_FLAG == 0 {
            cycles += 1;
            self.update_pc(operand as i8 as i32);
        } else {
            self.update_pc(opcode_len(mode));
        }
        self.update_cycles(cycles);
    }

    pub fn bpl(&mut self, mode: Mode) {
        let operand = self.lookup(mode);
        let mut cycles = 2;
        if self.state.status & N_FLAG == 0 {
            cycles += 1;
            self.update_pc(operand as i8 as i32);
        } else {
            self.update_pc(opcode_len(mode));
        }
        self.update_cycles(cycles);
    }

    pub fn clc(&mut self, mode: Mode) {
        self.state.status &= !C_FLAG;
        self.update_pc(opcode_len(mode)).update_cycles(2);
    }

    pub fn lda(&mut self, mode: Mode) {
        let operand = self.lookup(mode);

        self.set_reg(Reg::A, operand)
            .update_pc(opcode_len(mode))
            .update_status(operand, operand, operand, Z_FLAG | N_FLAG)
            .update_cycles(2);
    }

    pub fn sec(&mut self, mode: Mode) {
        self.state.status |= C_FLAG;
        self.update_pc(opcode_len(mode)).update_cycles(2);
    }

    pub fn nop(&mut self, mode: Mode) {
        println!("NOP");
        self.update_pc(opcode_len(mode)).update_cycles(1);
    }
}
