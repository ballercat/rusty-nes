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

const INSTRUCTION_STRINGS: &[&str] = &[
//           0:8    1:9    2:a    3:b    4:c    5:d    6:e    7:f               8    9      A      B      C      D      E      F
/* 0x00 */"BRK", "ORA", "JAM", "SLO", "NOP", "ORA", "ASL", "SLO",/* 0x08 */ "PHP", "ORA", "ASL", "ANC", "NOP", "ORA", "ASL", "SLO",
/* 0x10 */"BPL", "ORA", "JAM", "SLO", "NOP", "ORA", "ASL", "SLO",/* 0x18 */ "CLC", "ORA", "NOP", "SLO", "NOP", "ORA", "ASL", "SLO",
/* 0x20 */"JSR", "AND", "JAM", "RLA", "BIT", "AND", "ROL", "RLA",/* 0x28 */ "PLP", "AND", "ROL", "ANC", "BIT", "AND", "ROL", "RLA",
/* 0x30 */"BMI", "AND", "JAM", "RLA", "NOP", "AND", "ROL", "RLA",/* 0x38 */ "SEC", "AND", "NOP", "RLA", "NOP", "AND", "ROL", "RLA",
/* 0x40 */"RTI", "JAM", "JAM", "SRE", "NOP", "EOR", "LSR", "SRE",/* 0x48 */ "PHA", "EOR", "LSR", "ALR", "JMP", "EOR", "LSR", "SRE",
/* 0x50 */"BVC", "JAM", "JAM", "SRE", "NOP", "EOR", "LSR", "SRE",/* 0x58 */ "CLI", "EOR", "NOP", "SRE", "NOP", "EOR", "LSR", "SRE",
/* 0x60 */"RTS", "JAM", "JAM", "RRA", "NOP", "ADC", "ROR", "RRA",/* 0x68 */ "PLA", "ADC", "ROR", "ARR", "JMP", "ADC", "ROR", "RRA",
/* 0x70 */"BVS", "JAM", "JAM", "RRA", "NOP", "ADC", "ROR", "RRA",/* 0x78 */ "SEI", "ADC", "NOP", "RRA", "NOP", "ADC", "ROR", "RRA",
/* 0x80 */"NOP", "NOP", "NOP", "SAX", "STY", "STA", "STX", "SAX",/* 0x88 */ "DEY", "NOP", "TXA", "XAA", "STY", "STA", "STX", "SAx",
/* 0x90 */"BCC", "JAM", "JAM", "AHX", "STY", "STA", "STX", "SAX",/* 0x98 */ "TYA", "STA", "TXS", "TAS", "SHY", "STA", "SHX", "SHA",
/* 0xa0 */"LDY", "LDA", "LDX", "LAX", "LDY", "LDA", "LDX", "LAX",/* 0xA8 */ "TAY", "LDA", "TAX", "LAX", "LDY", "LDA", "LDX", "LAX",
/* 0xb0 */"BCS", "LDA", "JAM", "LAX", "LDY", "LDA", "LDX", "LAX",/* 0xB8 */ "CLV", "LDA", "TSX", "LAS", "LDY", "LDA", "LDX", "LAX",
/* 0xc0 */"CPY", "CMP", "NOP", "DCP", "CPY", "CMP", "DEC", "DCP",/* 0xC8 */ "INY", "CMP", "DEX", "AXS", "CPY", "CMP", "DEC", "DCP",
/* 0xd0 */"BNE", "CMP", "JAM", "DCP", "NOP", "CMP", "DEC", "DCP",/* 0xD8 */ "CLD", "CMP", "NOP", "DCP", "NOP", "CMP", "DEC", "DCP",
/* 0xe0 */"CPX", "SBC", "NOP", "ISC", "CPX", "SBC", "INC", "ISC",/* 0xE8 */ "INX", "SBC", "NOP", "SBC", "CPX", "SBC", "INC", "ISC",
/* 0xf0 */"BEQ", "SBC", "JAM", "ISC", "NOP", "SBC", "INC", "ISC",/* 0xF8 */ "SED", "SBC", "NOP", "ISC", "NOP", "SBC", "INC", "ISC",
];

pub fn opcode_name(opcode: u8) -> &'static str {
    let i : usize = opcode.into();
    INSTRUCTION_STRINGS[i]
}
