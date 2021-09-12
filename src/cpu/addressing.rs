use super::base::Processor;

#[derive(Copy, Clone)]
pub enum Mode {
    ZeroPage,
    Absolute,
    Accumulator,
    Immediate,
    Implied,
}

impl Processor {
    pub fn lookup(&mut self, mode: Mode) -> u8 {
        match mode {
            Mode::Absolute => {
                self.cycles += 2;
                let low = self.mem.read(self.state.pc + 1) as usize;
                let high = self.mem.read(self.state.pc + 2) as usize;
                let address = low | (high << 8);
                self.mem.read(address)
            }
            Mode::ZeroPage => {
                self.cycles += 1;
                let address = self.mem.read(self.state.pc + 1) as usize;
                self.mem.read(address)
            }
            Mode::Immediate => self.mem.read(self.state.pc + 1),
            Mode::Implied => {
                self.cycles += 1;
                0
            }
            Mode::Accumulator => self.state.a,
        }
    }

    pub fn get_byte_length(&mut self, mode: Mode) -> usize {
        match mode {
            Mode::Absolute => 3,
            Mode::ZeroPage | Mode::Immediate => 2,
            _ => 1,
        }
    }
}
