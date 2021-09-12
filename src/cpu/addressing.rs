use super::base::Processor;

#[derive(Copy, Clone)]
pub enum Mode {
    Accumulator,
    Immediate,
    Implied,
}

impl Processor {
    pub fn lookup(&mut self, mode: Mode) -> u8 {
        match mode {
            Mode::Immediate => self.mem.read(self.state.pc + 1),
            Mode::Implied => {
                self.cycles += 1;
                0
            }
            Mode::Accumulator => self.state.a,
        }
    }
}
