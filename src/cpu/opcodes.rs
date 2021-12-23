use super::addressing::Mode;
use super::base::{
    Processor, Reg, B_FLAG, C_FLAG, D_FLAG, F_FLAG, I_FLAG, N_FLAG, V_FLAG,
    Z_FLAG,
};
use regex::Regex;
use std::collections::HashMap;

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

pub const MODE_IML: u8 = 0b0000_0000;
pub const MODE_ZPG: u8 = 0b0000_0100;
pub const MODE_IMM: u8 = 0b0000_1000;
pub const MODE_ACC: u8 = 0b0000_1000;
pub const MODE_ABS: u8 = 0b0000_1100;
pub const MODE_IND: u8 = 0b0000_1100;
pub const MODE_INX: u8 = 0b0000_0000;
pub const MODE_INY: u8 = 0b0001_0000;
pub const MODE_REL: u8 = 0b0001_0000;
pub const MODE_ZPX: u8 = 0b0001_0100;
pub const MODE_ZPY: u8 = 0b0001_0100;
pub const MODE_ABY: u8 = 0b0001_1000;
pub const MODE_ABX: u8 = 0b0001_1100;

pub type Opcode = fn(&mut Processor, Mode) -> ();

lazy_static! {
    static ref OPCODE_HASHMAP: HashMap<&'static str, u8> = {
        let mut m = HashMap::new();
        m.insert("ADC", ADC);
        m.insert("AND", 0x29);
        m.insert("ASL", 0x06);
        m.insert("BCC", BCC);
        m.insert("BCS", BCS);
        m.insert("BEQ", BEQ);
        m.insert("BIT", BIT_Z);
        m.insert("BMI", BMI);
        m.insert("BNE", BNE);
        m.insert("BPL", BPL);
        m.insert("BRK", 0x00);
        m.insert("BVC", 0x50);
        m.insert("BVS", 0x70);
        m.insert("CLC", CLC);
        m.insert("SEC", SEC);
        m.insert("SED", 0xf8);
        m.insert("STA", 0x85);
        m.insert("NOP", NOP);
        m.insert("LDA", LDA);

        m
    };
}

pub fn opcode_len(mode: Mode) -> i32 {
    match mode {
        Mode::Absolute | Mode::AbsoluteX | Mode::AbsoluteY | Mode::Indirect => {
            3
        }
        Mode::ZeroPage
        | Mode::ZeroPageX
        | Mode::ZeroPageY
        | Mode::IndexedX
        | Mode::IndexedY
        | Mode::Relative
        | Mode::Immediate => 2,
        _ => 1,
    }
}

pub fn apply_address_mode(opcode: u8, mode: u8) -> u8 {
    // if the mode is implied then leave the raw opcode whatever it might be.
    // There are multiple instructions that use implied mode but do not share
    // the implied mode mask. In essence the mode has no impact on the opcode value
    if mode == MODE_IML {
        return opcode;
    }

    (opcode & 0b1110_0011) | mode
}

pub fn encode(line: &String) -> Vec<u8> {
    lazy_static! {
        static ref IMPLIED: Regex = Regex::new(r"^(?P<name>[A-Z]{3})[ ]*;.*$").unwrap();
        static ref ACCUMULATOR: Regex = Regex::new(r"^(?P<name>[A-Z]{3}) A[ ]*;.*$").unwrap();
        static ref ABSOLUTE: Regex = Regex::new(
            r"^(?P<name>[A-Z]{3}) \$(?P<low>[A-F0-9]{2})(?P<high>[A-F0-9]{2})[ ]*;.*$",
        )
        .unwrap();
        static ref ABSOLUTE_X: Regex = Regex::new(
            r"^(?P<name>[A-Z]{3}) \$(?P<low>[A-F0-9]{2})(?P<high>[A-F0-9]{2}),X[ ]*;.*$",
        )
        .unwrap();
        static ref ABSOLUTE_Y: Regex = Regex::new(
            r"^(?P<name>[A-Z]{3}) \$(?P<low>[A-F0-9]{2})(?P<high>[A-F0-9]{2}),Y[ ]*;.*$",
        )
        .unwrap();
        static ref IMMEDIATE: Regex =
            Regex::new(r"^(?P<name>[A-Z]{3}) #\$(?P<value>[A-F0-9]{2})[ ]*;.*$").unwrap();
        static ref INDIRECT: Regex =
            Regex::new(r"^(?P<name>[A-Z]{3}) \(\$(?P<low>[A-F0-9]{2})(?P<high>[A-F0-9]{2})\)[ ]*;.*$").unwrap();
        static ref X_INDEX: Regex = Regex::new(r"^(?P<name>[A-Z]{3}) \(\$(?P<value>[A-F0-9]{2}),X\)[ ]*;.*$").unwrap();
        static ref Y_INDEX: Regex = Regex::new(r"^(?P<name>[A-Z]{3}) \(\$(?P<value>[A-F0-9]{2})\),Y[ ]*;.*$").unwrap();
        static ref ZERO_PAGE: Regex =
            Regex::new(r"^(?P<name>[A-Z]{3}) \$(?P<value>[A-F0-9]{2})[ ]*;.*$").unwrap();
        static ref RELATIVE: Regex =
            Regex::new(r"^(?P<name>[A-Z]{3}) !\$(?P<value>[A-F0-9]{2})[ ]*;.*$").unwrap();
        static ref ZERO_PAGE_X: Regex =
            Regex::new(r"^(?P<name>[A-Z]{3}) \$(?P<value>[A-F0-9]{2}),X[ ]*;.*$").unwrap();
        static ref ZERO_PAGE_Y: Regex =
            Regex::new(r"^(?P<name>[A-Z]{3}) \$(?P<value>[A-F0-9]{2}),Y[ ]*;.*$").unwrap();
    }

    let apply_regex = |regex: &Regex, mode: u8| {
        let captures = regex.captures(line).unwrap();
        let opcode_value =
            *OPCODE_HASHMAP.get(&captures["name"]).unwrap_or(&NOP);
        let opcode = apply_address_mode(opcode_value, mode);
        let mut result: Vec<u8> = Vec::new();
        result.push(opcode);
        for cap in captures.iter().skip(2) {
            result
                .push(u8::from_str_radix(&cap.unwrap().as_str(), 16).unwrap());
        }
        if result.len() == 3 {
            result.swap(1, 2);
        }
        result
    };

    if ABSOLUTE.is_match(line) {
        apply_regex(&ABSOLUTE, MODE_ABS)
    } else if ACCUMULATOR.is_match(line) {
        apply_regex(&ACCUMULATOR, MODE_ACC)
    } else if ABSOLUTE_X.is_match(line) {
        apply_regex(&ABSOLUTE_X, MODE_ABX)
    } else if ABSOLUTE_Y.is_match(line) {
        apply_regex(&ABSOLUTE_Y, MODE_ABY)
    } else if IMMEDIATE.is_match(line) {
        apply_regex(&IMMEDIATE, MODE_IMM)
    } else if RELATIVE.is_match(line) {
        apply_regex(&RELATIVE, MODE_REL)
    } else if ZERO_PAGE.is_match(line) {
        apply_regex(&ZERO_PAGE, MODE_ZPG)
    } else if INDIRECT.is_match(line) {
        apply_regex(&INDIRECT, MODE_IND)
    } else if X_INDEX.is_match(line) {
        apply_regex(&X_INDEX, MODE_INX)
    } else if Y_INDEX.is_match(line) {
        apply_regex(&Y_INDEX, MODE_INY)
    } else if ZERO_PAGE_X.is_match(line) {
        apply_regex(&ZERO_PAGE_X, MODE_ZPX)
    } else if ZERO_PAGE_Y.is_match(line) {
        apply_regex(&ZERO_PAGE_Y, MODE_ZPY)
    } else {
        apply_regex(&IMPLIED, MODE_IML)
    }
}

impl Processor {
    pub fn decode(&self, value: u8) -> (Opcode, Mode) {
        // https://www.masswerk.at/6502/6502_instruction_set.html#layout
        let a = (value & 0b1110_0000) >> 5;
        let b = (value & 0b0001_1100) >> 2;
        let c = value & 0b0000_0011;

        match (c, b, a) {
            (0, 0, 0) => (Processor::brk, Mode::Implied),
            (0, 0, 1) => (Processor::jsr, Mode::Absolute),
            (0, 0, 2) => (Processor::rti, Mode::Implied),
            (0, 2, 0) => (Processor::php, Mode::Implied),
            (0, 2, 1) => (Processor::plp, Mode::Implied),
            (0, 2, 2) => (Processor::pha, Mode::Implied),
            (0, 2, 3) => (Processor::pla, Mode::Implied),
            (0, 1, 1) => (Processor::bit, Mode::ZeroPage),
            (0, 3, 1) => (Processor::bit, Mode::Absolute),
            // Branches
            (0, 4, _) => {
                let instruction = match a {
                    0 => Processor::bpl,
                    1 => Processor::bmi,
                    2 => Processor::bvc,
                    3 => Processor::bvs,
                    4 => Processor::bcc,
                    5 => Processor::bcs,
                    6 => Processor::bne,
                    7 => Processor::beq,
                    _ => panic!("Cannot decode instruction: {}", value),
                };
                (instruction, Mode::Relative)
            }
            (0, 6, 0) => (Processor::clc, Mode::Implied),
            (0, 6, 6) => (Processor::cld, Mode::Implied),
            (0, 6, 1) => (Processor::sec, Mode::Implied),
            (1, _, _) => {
                let mode = match b {
                    0 => Mode::Indirect,
                    1 => Mode::ZeroPage,
                    2 => Mode::Immediate,
                    3 => Mode::Absolute,
                    4 => Mode::Indirect,
                    5 => Mode::ZeroPageX,
                    6 => Mode::AbsoluteX,
                    7 => Mode::AbsoluteY,
                    _ => panic!("Cannot decode instruction: {}", value),
                };

                let instruction = match a {
                    4 => Processor::sta,
                    1 => Processor::and,
                    3 => Processor::adc,
                    5 => Processor::lda,
                    _ => Processor::nop,
                };

                (instruction, mode)
            }
            (2, 2, 0) => (Processor::asl, Mode::Accumulator),
            (2, 2, 7) => (Processor::nop, Mode::Implied),
            _ => (Processor::nop, Mode::Implied),
        }
    }

    pub fn adc(&mut self, mode: Mode) {
        let address = self.lookup(mode);
        let operand = self.mem.read(address);
        let accumulator = self.state.a;
        let carry = self.state.status & 1;
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
        let address = self.lookup(mode);
        let operand = self.mem.read(address);
        let accumulator = self.get_reg(Reg::A);
        let result = accumulator & operand;
        self.set_reg(Reg::A, result)
            .update_pc(opcode_len(mode))
            .update_status(accumulator, operand, result, N_FLAG | Z_FLAG)
            .update_cycles(2);
    }

    pub fn asl(&mut self, mode: Mode) {
        let address = self.lookup(mode);
        // FIXME: this isn't ideal when mode is accumulator the logic is altered heavily
        let operand = match mode {
            Mode::Accumulator => self.state.a,
            _ => self.mem.read(address),
        };
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
        if self.state.status & C_FLAG == 0 {
            // Jump location lookup costs cycles but these are "free" if the
            // jump will not occur. That's why the lookup must be done AFTER
            // checking the condition above. This is true for all branch opcodes
            let address = self.lookup(mode);
            self.jump(address);
        } else {
            self.update_pc(opcode_len(mode));
        }

        self.update_cycles(2);
    }

    pub fn bcs(&mut self, mode: Mode) {
        if self.state.status & C_FLAG != 0 {
            let address = self.lookup(mode);
            self.jump(address);
        } else {
            self.update_pc(opcode_len(mode));
        }
        self.update_cycles(2);
    }

    pub fn beq(&mut self, mode: Mode) {
        if self.state.status & Z_FLAG != 0 {
            let address = self.lookup(mode);
            self.jump(address);
        } else {
            self.update_pc(opcode_len(mode));
        }
        self.update_cycles(2);
    }

    pub fn bit(&mut self, mode: Mode) {
        let address = self.lookup(mode);
        let operand = self.mem.read(address);
        let accumulator = self.state.a;
        let result = operand & accumulator;

        let new_flags = operand & (N_FLAG | V_FLAG);
        self.state.status =
            (self.state.status & !(N_FLAG | V_FLAG)) | new_flags;

        self.update_status(accumulator, operand, result, Z_FLAG)
            .update_pc(opcode_len(mode));
    }

    pub fn bmi(&mut self, mode: Mode) {
        if self.state.status & N_FLAG != 0 {
            let address = self.lookup(mode);
            self.jump(address);
        } else {
            self.update_pc(opcode_len(mode));
        }
        self.update_cycles(2);
    }

    pub fn bne(&mut self, mode: Mode) {
        if self.state.status & Z_FLAG == 0 {
            let address = self.lookup(mode);
            self.jump(address);
        } else {
            self.update_pc(opcode_len(mode));
        }
        self.update_cycles(2);
    }

    pub fn bpl(&mut self, mode: Mode) {
        if self.state.status & N_FLAG == 0 {
            let address = self.lookup(mode);
            self.jump(address);
        } else {
            self.update_pc(opcode_len(mode));
        }
        self.update_cycles(2);
    }

    pub fn brk(&mut self, _mode: Mode) {
        self.stack_push(self.state.status | F_FLAG | B_FLAG);
        self.state.status |= I_FLAG;

        println!("BRK not yet implemented");
    }

    pub fn bvc(&mut self, mode: Mode) {
        if self.state.status & V_FLAG == 0 {
            let address = self.lookup(mode);
            self.jump(address);
        } else {
            self.update_pc(opcode_len(mode));
        }
        self.update_cycles(2);
    }

    pub fn bvs(&mut self, mode: Mode) {
        if self.state.status & V_FLAG == 1 {
            let address = self.lookup(mode);
            self.jump(address);
        } else {
            self.update_pc(opcode_len(mode));
        }
        self.update_cycles(2);
    }

    pub fn clc(&mut self, mode: Mode) {
        self.state.status &= !C_FLAG;
        self.update_pc(opcode_len(mode)).update_cycles(2);
    }

    pub fn cld(&mut self, mode: Mode) {
        self.state.status &= !D_FLAG;
        self.update_pc(opcode_len(mode)).update_cycles(2);
    }

    pub fn jsr(&mut self, mode: Mode) {
        let address = self.lookup(mode);
        let pch = self.state.pc >> 8;
        let pcl = self.state.pc & 0xff;

        self.stack_push(pch as u8);
        self.stack_push(pcl as u8);

        self.jump(address).update_cycles(4);
    }

    pub fn lda(&mut self, mode: Mode) {
        let address = self.lookup(mode);
        let operand = self.mem.read(address);

        self.set_reg(Reg::A, operand)
            .update_pc(opcode_len(mode))
            .update_status(operand, operand, operand, Z_FLAG | N_FLAG)
            .update_cycles(2);
    }

    pub fn pha(&mut self, mode: Mode) {
        self.stack_push(self.state.a);

        self.update_pc(opcode_len(mode)).update_cycles(2);
    }

    pub fn php(&mut self, mode: Mode) {
        // https://wiki.nesdev.org/w/index.php?title=Status_flags
        // bit 5 & 4 of the status byte pushed onto the stack must be set
        // without having a side-effect on the contents of status itself
        self.stack_push(self.state.status | B_FLAG | F_FLAG);
        self.update_pc(opcode_len(mode)).update_cycles(2);
    }

    pub fn pla(&mut self, mode: Mode) {
        self.state.a = self.stack_pop();
        self.update_pc(opcode_len(mode)).update_cycles(3);
    }

    pub fn plp(&mut self, mode: Mode) {
        self.state.status = self.stack_pop();
        self.update_pc(opcode_len(mode)).update_cycles(3);
    }

    pub fn rti(&mut self, _mode: Mode) {
        // FIXME: break flag & bit 5 should be ignored from the pop-ed status
        let status = self.stack_pop();
        let pcl = self.stack_pop() as usize;
        let pch = self.stack_pop() as usize;
        let new_pc = pcl & (pch << 8);

        self.state.status = status;
        self.jump(new_pc).update_cycles(6);
    }

    pub fn sec(&mut self, mode: Mode) {
        self.state.status |= C_FLAG;
        self.update_pc(opcode_len(mode)).update_cycles(2);
    }

    pub fn sta(&mut self, mode: Mode) {
        let address = self.lookup(mode);
        self.mem.write(address, self.get_reg(Reg::A));
        self.update_pc(opcode_len(mode)).update_cycles(2);
    }

    pub fn nop(&mut self, mode: Mode) {
        println!("NOP");
        self.update_pc(opcode_len(mode)).update_cycles(1);
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_encode() {
        let program = encode(&String::from("ADC;"));
        assert_eq!(program[0], apply_address_mode(ADC, MODE_IML));

        // test comments
        let program = encode(&String::from("ADC; this is a comment"));
        assert_eq!(program[0], apply_address_mode(ADC, MODE_IML));

        let program = encode(&String::from(
            "ADC     ;semi-colon can be spaced however needed",
        ));
        assert_eq!(program[0], apply_address_mode(ADC, MODE_IML));

        let program = encode(&String::from("ADC #$A0;"));
        assert_eq!(program[0], apply_address_mode(ADC, MODE_IMM));
        assert_eq!(program[1], 0xa0);

        let program = encode(&String::from("ADC $A0;"));
        assert_eq!(program[0], apply_address_mode(ADC, MODE_ZPG));
        assert_eq!(program[1], 0xa0);

        let program = encode(&String::from("ADC $A0,X;"));
        assert_eq!(program[0], apply_address_mode(ADC, MODE_ZPX));
        assert_eq!(program[1], 0xa0);

        let program = encode(&String::from("ADC $A0,Y;"));
        assert_eq!(program[0], apply_address_mode(ADC, MODE_ZPY));
        assert_eq!(program[1], 0xa0);

        let program = encode(&String::from("ADC $A0FF;"));
        assert_eq!(program[0], apply_address_mode(ADC, MODE_ABS));
        assert_eq!(program[1], 0xff);
        assert_eq!(program[2], 0xa0);

        let program = encode(&String::from("ADC $A0FF,X;"));
        assert_eq!(program[0], apply_address_mode(ADC, MODE_ABX));
        assert_eq!(program[1], 0xff);
        assert_eq!(program[2], 0xa0);

        let program = encode(&String::from("ADC $A0FF,Y;"));
        assert_eq!(program[0], apply_address_mode(ADC, MODE_ABY));
        assert_eq!(program[1], 0xff);
        assert_eq!(program[2], 0xa0);

        // indirect instruction encoding. Note that ADC does not actually have an indirect
        // version on the real cpu. This is for testing purposes only.
        let program = encode(&String::from("ADC ($AABB);"));
        assert_eq!(program[0], apply_address_mode(ADC, MODE_IND));
        assert_eq!(program[1], 0xbb);
        assert_eq!(program[2], 0xaa);

        let program = encode(&String::from("ADC ($AA,X);"));
        assert_eq!(program[0], apply_address_mode(ADC, MODE_INX));
        assert_eq!(program[1], 0xaa);

        let program = encode(&String::from("ADC ($BB),Y;"));
        assert_eq!(program[0], apply_address_mode(ADC, MODE_INY));
        assert_eq!(program[1], 0xbb);
    }
}
