use super::base::Processor;

#[derive(Copy, Clone, Debug)]
pub enum Mode {
    ZeroPage,
    Absolute,
    Accumulator,
    Immediate,
    Implied,
    Indirect,
    IndexedY,
    IndexedX,
    ZeroPageX,
    ZeroPageY,
    AbsoluteX,
    AbsoluteY,
    Relative,
}

impl Processor {
    pub fn lookup(&mut self, mode: Mode) -> usize {
        match mode {
            Mode::Accumulator => self.state.a as usize,
            Mode::Absolute => {
                self.cycles += 2;
                let high = self.mem.read(self.state.pc + 1) as usize;
                let low = self.mem.read(self.state.pc + 2) as usize;
                low | (high << 8)
            }
            Mode::AbsoluteX => {
                self.cycles += 2;
                let carry = self.state.status & 1;
                let high = self.mem.read(self.state.pc + 1) as usize;
                let low = self.mem.read(self.state.pc + 2) as usize;
                let address = (low | (high << 8))
                    + carry as u32 as usize
                    + self.state.x as u32 as usize;
                if address >> 8 > high {
                    self.cycles += 1;
                }
                address
            }
            Mode::AbsoluteY => {
                self.cycles += 2;
                let carry = self.state.status & 1;
                let high = self.mem.read(self.state.pc + 1) as usize;
                let low = self.mem.read(self.state.pc + 2) as usize;
                let address = (low | (high << 8))
                    + carry as u32 as usize
                    + self.state.y as u32 as usize;
                if address >> 8 > high {
                    self.cycles += 1;
                }
                address
            }
            Mode::Immediate => (self.state.pc + 1) as usize,
            Mode::Implied => {
                self.cycles += 1;
                0
            }
            Mode::Indirect => {
                self.cycles += 5;
                let high = self.mem.read(self.state.pc + 1) as usize;
                let low = self.mem.read(self.state.pc + 2) as usize;
                self.mem.read(low | (high << 8)) as usize
            }
            Mode::IndexedX => {
                self.cycles += 4;
                let base_index =
                    (self.mem.read(self.state.pc + 1) + self.state.x) as usize;
                let high = self.mem.read(base_index) as usize;
                let low = self.mem.read(base_index + 1) as usize;
                low | (high << 8)
            }
            Mode::IndexedY => {
                // by default 3 cycles
                self.cycles += 3;
                let carry = self.state.status & 1;
                let base_index = self.mem.read(self.state.pc + 1) as usize;
                let high = self.mem.read(base_index) as usize;
                let low = self.mem.read(base_index + 1) as usize;
                let address = (low | (high << 8))
                    + carry as u32 as usize
                    + self.state.y as u32 as usize;
                // If page boundary is crossed IE. high byte is incremented at all
                // then add a cycle
                if address >> 8 > high {
                    self.cycles += 1;
                }
                address
            }
            Mode::Relative => {
                self.cycles += 1;
                // Read as i8 is important as a negative 8 bit value will fit
                // into a 32 bit signed integer and become a positive
                let offset = self.mem.read(self.state.pc + 1) as i8 as i32;
                let address = if offset.is_negative() {
                    self.state.pc - offset.wrapping_abs() as usize
                } else {
                    self.state.pc + offset as usize
                };
                // Crossing a page boundary with a jump will cause an extra cycle
                if address >> 8 > self.state.pc >> 8 {
                    self.cycles += 1;
                }
                address
            }
            Mode::ZeroPage => {
                self.cycles += 1;
                self.mem.read(self.state.pc + 1) as usize
            }
            Mode::ZeroPageX => {
                self.cycles += 2;
                let address = (0xFF
                    & (self.mem.read(self.state.pc + 1) + self.state.x))
                    as usize;
                address
            }
            Mode::ZeroPageY => {
                self.cycles += 2;
                let address = (0xff
                    & (self.mem.read(self.state.pc + 1) + self.state.y))
                    as usize;
                address
            }
        }
    }
}
